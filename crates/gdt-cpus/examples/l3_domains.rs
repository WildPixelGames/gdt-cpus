//! L3 Cache Domain Ping-Pong Benchmark
//!
//! Demonstrates WHY the L3-domain table exists: core-to-core communication
//! latency is dramatically different WITHIN an L3 domain (one CCD on chiplet
//! AMD, one cluster on hybrid Intel) than ACROSS domains, where every bounce
//! of the shared cache line crosses the die fabric.
//!
//! Two threads ping-pong a single cache-line-padded atomic flag, pinned to:
//!   1. two SMT siblings of one core      (same L1/L2 - the floor)
//!   2. two cores in the SAME L3 domain   (cache line bounces within the CCD)
//!   3. two cores in DIFFERENT L3 domains (cache line crosses the fabric)
//!
//! On a dual-CCD part (e.g. Ryzen 5950X) expect a multi-x latency cliff
//! between (2) and (3). That cliff is the reason cooperating threads - a
//! physics pair, producer/consumer queues, a job-stealing pool - should be
//! placed with `CpuInfo::l3_domain_mask()`, not scattered by the scheduler.

use gdt_cpus::{CpuInfo, Lp, pin_thread_to_core};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU32, Ordering},
};
use std::thread;
use std::time::Instant;

const ROUND_TRIPS: u64 = 1_000_000;

/// One atomic on its own cache line (and its own sector pair - 128 covers
/// adjacent-line prefetchers) so the ping-pong measures coherency traffic,
/// not false sharing.
#[repr(align(128))]
struct PaddedFlag(AtomicU32);

/// Bounces the flag between two pinned threads; returns ns per round trip.
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

/// First primary thread (smt_index == 0) of each physical core in a domain.
fn primary_lps_of_domain(info: &CpuInfo, domain: u8) -> Vec<&Lp> {
    info.lps
        .iter()
        .filter(|lp| lp.smt_index == 0 && lp.l3_domain == domain)
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = CpuInfo::detect()?;

    println!(
        "CPU: {} - {} cores / {} threads, {} L3 domain(s):",
        info.model_name,
        info.core_count,
        info.lps.len(),
        info.l3_domains.len()
    );
    for (i, d) in info.l3_domains.iter().enumerate() {
        println!(
            "  domain {}: {:>5} MiB, {:>2} cores, {:>2} threads",
            i,
            d.size_bytes / (1024 * 1024),
            d.core_count,
            d.mask.count()
        );
    }
    println!("\n{} round trips per configuration...\n", ROUND_TRIPS);

    // 1. SMT siblings of one core (if SMT exists): the communication floor.
    let smt_pair = info.lps.iter().find(|lp| lp.smt_index == 1).map(|sibling| {
        let primary = info
            .lps
            .iter()
            .find(|lp| lp.core == sibling.core && lp.smt_index == 0)
            .expect("SMT sibling without a primary thread");
        (primary.os_id as usize, sibling.os_id as usize)
    });
    if let Some((a, b)) = smt_pair {
        let ns = ping_pong(a, b);
        println!(
            "SMT siblings, one core      (lp {:>3} <-> lp {:>3}): {:>8.1} ns/round-trip",
            a, b, ns
        );
    } else {
        println!("SMT siblings, one core      : no SMT on this machine, skipped");
    }

    // 2. Two cores within the first L3 domain.
    let domain0 = primary_lps_of_domain(&info, 0);
    let same_domain_ns = if domain0.len() >= 2 {
        let (a, b) = (domain0[0].os_id as usize, domain0[1].os_id as usize);
        let ns = ping_pong(a, b);
        println!(
            "Two cores, SAME L3 domain   (lp {:>3} <-> lp {:>3}): {:>8.1} ns/round-trip",
            a, b, ns
        );
        Some(ns)
    } else {
        println!("Two cores, SAME L3 domain   : domain 0 has a single core, skipped");
        None
    };

    // 3. Two cores in different L3 domains - the fabric crossing.
    if info.l3_domains.len() >= 2 {
        let domain1 = primary_lps_of_domain(&info, 1);
        if let (Some(a), Some(b)) = (domain0.first(), domain1.first()) {
            let (a, b) = (a.os_id as usize, b.os_id as usize);
            let ns = ping_pong(a, b);
            println!(
                "Two cores, CROSS L3 domains (lp {:>3} <-> lp {:>3}): {:>8.1} ns/round-trip",
                a, b, ns
            );
            if let Some(same) = same_domain_ns {
                println!(
                    "\nCrossing the L3 fabric costs {:.1}x an in-domain round trip; pin cooperating threads with l3_domain_mask().",
                    ns / same
                );
            }
        }
    } else {
        println!("Two cores, CROSS L3 domains : single L3 domain on this machine, skipped");
        println!("\n(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -");
        println!("to see the cross-domain latency cliff the L3 table exists for.)");
    }

    Ok(())
}
