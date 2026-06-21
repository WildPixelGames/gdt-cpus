//! Packing N cooperating cores into the tightest cache neighborhood.
//!
//! A job system wants K workers placed as physically close as possible - a
//! producer/consumer pair, a parallel-for over shared data, a physics island.
//! Within one L3 domain, `CpuInfo::l2_domains` (filtered by
//! [`L2Domain::l3_domain`](gdt_cpus::L2Domain::l3_domain)) lists the L2 sharing
//! groups in topology order. Taking WHOLE L2 groups in order packs the workers
//! onto adjacent cores; grabbing arbitrary core ids scatters them across the L3,
//! paying more cross-core coherency traffic.
//!
//! How much that buys you is hardware-shaped:
//!   - hybrid Intel E-cores share one L2 in clusters of 4, so packing keeps a
//!     worker set inside a single L2 - the tightest sharing the chip offers;
//!   - most desktop AMD give each physical core its own L2, so the L2 groups are
//!     single cores and packing is just "the first K cores in topology order" -
//!     still the right answer, there is simply no intra-L3 L2 cliff to measure
//!     (the cliff there is between L3 domains - see `l3_domains.rs`);
//!   - Apple Silicon has no L3 at all: each performance/efficiency cluster shares
//!     one L2, which is then the coarsest cache grouping, so packing keeps a set
//!     inside one cluster.
//!
//! This example picks the busiest L3 domain (or, on no-L3 parts, all L2 groups),
//! packs K cores out of it via L2 groups, then measures the tightest core pair it
//! offers against the next coherency boundary out - same-L2 vs cross-L2 where
//! cores share an L2, in-L3 vs cross-L3 where each core owns one - so the payoff
//! is a measured ratio, not an asserted moral.

use gdt_cpus::{AffinityMask, CpuInfo, Lp, pin_thread_to_core};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU32, Ordering},
};
use std::thread;
use std::time::Instant;

const ROUND_TRIPS: u64 = 1_000_000;

/// One atomic on its own cache line (+ sector pair) so the ping-pong measures
/// coherency traffic, not false sharing.
#[repr(align(128))]
struct PaddedFlag(AtomicU32);

/// Bounces a flag between two pinned threads; returns ns per round trip.
fn ping_pong(lp_a: usize, lp_b: usize) -> f64 {
    let flag = Arc::new(PaddedFlag(AtomicU32::new(0)));
    let barrier = Arc::new(Barrier::new(2));

    let ponger = {
        let flag = Arc::clone(&flag);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            pin_thread_to_core(lp_b).expect("failed to pin ponger");
            barrier.wait();
            for _ in 0..ROUND_TRIPS {
                while flag.0.load(Ordering::Acquire) != 1 {
                    std::hint::spin_loop();
                }
                flag.0.store(0, Ordering::Release);
            }
        })
    };

    let pinger = {
        let flag = Arc::clone(&flag);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            pin_thread_to_core(lp_a).expect("failed to pin pinger");
            barrier.wait();
            let start = Instant::now();
            for _ in 0..ROUND_TRIPS {
                flag.0.store(1, Ordering::Release);
                while flag.0.load(Ordering::Acquire) != 0 {
                    std::hint::spin_loop();
                }
            }
            start.elapsed()
        })
    };

    let elapsed = pinger.join().expect("pinger panicked");
    ponger.join().expect("ponger panicked");
    elapsed.as_nanos() as f64 / ROUND_TRIPS as f64
}

/// Primary (`smt_index == 0`) OS ids of L2 domain `idx`, in ascending order -
/// one worker slot per physical core sharing that L2.
fn primary_cores_of_l2(info: &CpuInfo, idx: usize) -> Vec<usize> {
    info.l2_domains[idx]
        .mask
        .iter()
        .filter(|&os_id| {
            info.lps
                .iter()
                .any(|lp| lp.os_id as usize == os_id && lp.smt_index == 0)
        })
        .collect()
}

/// The L2 domains of L3 domain `l3`, in topology order, as the index into
/// `info.l2_domains`.
fn l2_domains_of_l3(info: &CpuInfo, l3: u8) -> Vec<usize> {
    info.l2_domains
        .iter()
        .enumerate()
        .filter(|(_, d)| d.l3_domain == l3)
        .map(|(i, _)| i)
        .collect()
}

/// Packs `k` primary cores out of L3 domain `l3` by taking whole L2 groups in
/// topology order - the placement a cache-aware pool wants.
fn pack_closest(info: &CpuInfo, l3: u8, k: usize) -> AffinityMask {
    let mut picked = Vec::new();
    'groups: for idx in l2_domains_of_l3(info, l3) {
        for os_id in primary_cores_of_l2(info, idx) {
            picked.push(os_id);
            if picked.len() == k {
                break 'groups;
            }
        }
    }
    AffinityMask::from_cores(&picked)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = CpuInfo::detect()?;

    println!(
        "CPU: {} - {} cores / {} threads, {} L3 / {} L2 domain(s)",
        info.model_name,
        info.core_count,
        info.lps.len(),
        info.l3_domains.len(),
        info.l2_domains.len(),
    );

    if info.l2_domains.is_empty() {
        println!("\nNo L2 domains detected on this machine (cache topology unavailable)");
        println!("- nothing to pack. See `basic_info`.");
        return Ok(());
    }

    // The "scope" to slice from is the coarsest cache neighborhood the chip has:
    // the busiest L3 domain on parts that have L3 (most x86), or the whole set
    // of L2 groups on no-L3 parts (Apple Silicon - the L2 cluster IS the coarsest
    // sharing, and `l3_domains` is empty, so indexing it would panic). `l3` is
    // `NO_L3` in the no-L3 case, which `l2_domains_of_l3`/`pack_closest` match
    // against the NO_L3-tagged domains.
    let has_l3 = !info.l3_domains.is_empty();
    let (l3, scope_cores, groups, scope_label) = if has_l3 {
        let l3 = (0..info.l3_domains.len() as u8)
            .max_by_key(|&d| info.l3_domains[d as usize].core_count)
            .unwrap_or(0);
        let cores = info.l3_domains[l3 as usize].core_count as usize;
        (
            l3,
            cores,
            l2_domains_of_l3(&info, l3),
            format!("L3 domain {l3}"),
        )
    } else {
        let cores = info.l2_domains.iter().map(|d| d.core_count as usize).sum();
        let groups = (0..info.l2_domains.len()).collect();
        (Lp::NO_L3, cores, groups, "no-L3 machine".to_string())
    };

    println!(
        "\n{scope_label} ({scope_cores} cores) splits into {} L2 group(s):",
        groups.len()
    );
    let idx_w = groups.iter().max().copied().unwrap_or(0).to_string().len();
    for &idx in &groups {
        println!(
            "  L2 {idx:>idx_w$}: {} KB, cores {}",
            info.l2_domains[idx].size_bytes / 1024,
            info.l2_domains[idx].mask,
        );
    }

    let k = scope_cores.min(4);
    let packed = pack_closest(&info, l3, k);
    println!("\nPacking {k} cooperating cores via L2 groups: {packed}");

    // Measure the tightest core pair the chip offers against the next coherency
    // boundary out, both chosen from the detected topology - so the closing
    // number is read from this run, not asserted. Where cores share an L2 the
    // tier is same-L2 vs same-L3-different-L2; where each core owns its L2 the
    // floor is in-L3 and the next cliff is the L3 fabric.
    let shares_l2 = groups
        .iter()
        .any(|&idx| primary_cores_of_l2(&info, idx).len() >= 2);
    let l3_primaries: Vec<usize> = groups
        .iter()
        .flat_map(|&idx| primary_cores_of_l2(&info, idx))
        .collect();

    let widest = groups
        .iter()
        .map(|&idx| primary_cores_of_l2(&info, idx))
        .max_by_key(|cores| cores.len())
        .unwrap_or_default();

    let tight = if shares_l2 {
        ("tightest, SAME L2", widest[0], widest[1])
    } else if l3_primaries.len() >= 2 {
        let label = if has_l3 {
            "tightest, SAME L3 (own L2)"
        } else {
            "tightest, own L2 each"
        };
        (label, l3_primaries[0], l3_primaries[1])
    } else {
        println!("\n{scope_label} has a single core - no in-domain round trip to measure.");
        return Ok(());
    };

    let wide = if shares_l2 {
        let label = if has_l3 {
            "next out, same L3, diff L2"
        } else {
            "next out, diff L2 cluster"
        };
        groups
            .iter()
            .map(|&idx| primary_cores_of_l2(&info, idx))
            .find(|c| !c.is_empty() && c[0] != tight.1 && c[0] != tight.2)
            .map(|c| (label, tight.1, c[0]))
    } else {
        info.lps
            .iter()
            .find(|lp| lp.smt_index == 0 && lp.l3_domain != l3 && lp.l3_domain != Lp::NO_L3)
            .map(|lp| ("next out, CROSS L3 domain", tight.1, lp.os_id as usize))
    };

    println!("\n{ROUND_TRIPS} round trips per configuration...\n");
    let tight_ns = ping_pong(tight.1, tight.2);
    println!(
        "{:<26} (lp {:>3} <-> lp {:>3}): {tight_ns:>8.1} ns/round-trip",
        tight.0, tight.1, tight.2
    );

    if let Some((wlabel, wa, wb)) = wide {
        let wide_ns = ping_pong(wa, wb);
        println!("{wlabel:<26} (lp {wa:>3} <-> lp {wb:>3}): {wide_ns:>8.1} ns/round-trip");
        let scope = if shares_l2 {
            "whole L2 groups"
        } else {
            "one L3 domain (l3_domain_mask)"
        };
        println!(
            "\nThe next boundary out costs {:.1}x the tightest pair; pack cooperating workers into {scope}.",
            wide_ns / tight_ns,
        );
    } else {
        println!(
            "\nOnly one coherency scope here: {tight_ns:.1} ns floor, no coarser boundary to cross."
        );
    }

    Ok(())
}
