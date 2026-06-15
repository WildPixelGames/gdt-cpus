//! reserved_core - for a latency-critical thread, compare placement vs priority.
//!
//! A 48 kHz audio feeder must hit a 5333 us deadline every buffer while the rest
//! of the engine saturates the other P-cores with Highest-priority spinners. Two
//! ways to protect the feeder are compared:
//!
//!   1. reserved core @ Normal - feeder alone on its own P-core, ordinary
//!      priority; no spinner shares it.
//!   2. contended core @ TimeCritical - feeder on a core a spinner already owns,
//!      handed the strongest priority this crate offers, fighting for the slice.
//!
//! The result is deliberately computed from the run. Some schedulers let the
//! strongest priority dominate this synthetic shared-core fight; others still
//! charge a steady time-slice tax. The point is to measure which lever buys the
//! better tail on this OS instead of hardcoding the folklore.
//!
//! NOTE: the magnitude and even the winner are scheduler- and box-dependent
//! (nice weights, Windows priority boosts, CFS granularity, whether negative nice
//! is grantable here at all). The portable lesson is the comparison, not a fixed
//! ordering.
//!
//! NOTE: the spinner pool is spawned ONCE on the other P-cores and reused by
//! every run. rtkit's per-UID grant budget is small (a couple dozen actions in a
//! rolling window); respawning Highest spinners each run would exhaust it and
//! silently drop later threads to Normal, poisoning the contention the demo needs.

mod support;

use gdt_cpus::{
    AppliedPriority, CoreKind, CpuInfo, ThreadPriority, pin_thread_to_core, set_thread_priority,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

const SAMPLE_RATE: u64 = 48_000;
const BUFFER_SAMPLES: u64 = 256;
const BUFFER_US: u64 = (BUFFER_SAMPLES * 1_000_000) / SAMPLE_RATE; // ~5333 us
const TEST_SECS: u64 = 3;
const HARD_CAP: Duration = Duration::from_secs(TEST_SECS + 3);
const CONTENDED_RUNS: usize = 5;

#[inline]
fn cpu_relax() {
    std::hint::spin_loop();
}

/// One Highest-priority spinner per id in `lps` - the rest of the engine holding
/// those cores at 100%.
fn spawn_load(
    stop: Arc<AtomicBool>,
    lps: &[usize],
) -> (
    Vec<thread::JoinHandle<()>>,
    Vec<gdt_cpus::Result<AppliedPriority>>,
) {
    let (tx, rx) = mpsc::channel();
    let handles = lps
        .iter()
        .map(|&lp| {
            let stop = stop.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                let priority_result = set_thread_priority(ThreadPriority::Highest);
                let _ = tx.send(priority_result.clone());
                let _ = pin_thread_to_core(lp);
                while !stop.load(Ordering::SeqCst) {
                    cpu_relax();
                }
            })
        })
        .collect::<Vec<_>>();
    drop(tx);
    let results = rx.into_iter().take(handles.len()).collect();
    (handles, results)
}

/// Pin the feeder to `audio_core`, set `priority`, spin-wait to each 5333 us
/// deadline and record the overshoot. Spawns ONLY the feeder; the load pool is
/// already running. Returns (applied-priority string, fell_back?, jitter us).
///
/// `fell_back` is `requested != effective` - the NAMED level actually dropped
/// (e.g. TimeCritical -> Normal), which would make the contention numbers a
/// starve. A within-level clamp (TimeCritical -> nice -15) is NOT a fallback: the
/// feeder still ran strong, so `degraded()` would over-report here.
///
/// NOTE: the feeder spin-waits instead of sleeping, so the jitter is preemption
/// overshoot of a 100%-CPU thread on a contended core, not sleep-wakeup latency.
fn run_feeder(audio_core: usize, priority: ThreadPriority) -> (String, bool, Vec<u64>) {
    thread::spawn(move || {
        let _ = pin_thread_to_core(audio_core);
        let priority_result = set_thread_priority(priority);
        let fell_back = priority_result
            .as_ref()
            .map_or(true, |applied| applied.requested() != applied.effective());
        let technique = support::priority_detail(priority, &priority_result);

        let expected = (TEST_SECS * 1_000_000 / BUFFER_US) as usize;
        let mut lat = Vec::with_capacity(expected);
        let cap = Instant::now() + HARD_CAP;
        let mut last = Instant::now();
        while Instant::now() < cap && lat.len() < expected {
            let target = last + Duration::from_micros(BUFFER_US);
            loop {
                let now = Instant::now();
                if now >= target {
                    lat.push(now.duration_since(target).as_micros() as u64);
                    last = now;
                    break;
                }
                if now >= cap {
                    break;
                }
                cpu_relax();
            }
        }
        (technique, fell_back, lat)
    })
    .join()
    .unwrap()
}

fn pctl(mut data: Vec<u64>, pct: usize) -> u64 {
    if data.is_empty() {
        return 0;
    }
    data.sort_unstable();
    data[data.len() * pct / 100]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = CpuInfo::detect()?;

    // Primary thread (smt_index == 0) of each Performance core, by OS LP id.
    let p_lps: Vec<usize> = info
        .lps
        .iter()
        .filter(|lp| lp.smt_index == 0 && lp.kind == CoreKind::Performance)
        .map(|lp| lp.os_id as usize)
        .collect();
    if p_lps.len() < 2 {
        eprintln!("need at least 2 Performance-core primaries for this demo");
        return Ok(());
    }

    let audio_core = p_lps[0]; // reserved: no spinner pinned here
    let contended_core = p_lps[1]; // shared: a spinner owns this one
    let load_lps = &p_lps[1..]; // every P-core EXCEPT audio_core

    println!("{} - {} P-core primaries", info.model_name, p_lps.len());
    println!(
        "Feeder: 48 kHz / 256-frame buffer ({BUFFER_US}us deadline). \
         Load: {} Highest spinners on the other P-cores, spawned once.\n",
        load_lps.len()
    );

    // Spawn the spinner pool ONCE (see module NOTE on the rtkit grant budget).
    let stop = Arc::new(AtomicBool::new(false));
    let (load, load_priority_results) = spawn_load(stop.clone(), load_lps);
    let mut spinner_tally = support::PriorityTally::new(ThreadPriority::Highest);
    for result in load_priority_results {
        spinner_tally.record(result);
    }
    println!("Load spinner priority: {}", spinner_tally.render());

    println!("Lower jitter is better. Which lever buys it - own core, or top priority?\n");

    // 1. Reserved: ordinary priority, but audio_core has no spinner -> deterministic.
    let (tech, _deg, lat) = run_feeder(audio_core, ThreadPriority::Normal);
    let r_p50 = pctl(lat.clone(), 50);
    let r_p95 = pctl(lat.clone(), 95);
    let r_p99 = pctl(lat, 99);
    println!(
        "  reserved core  @ Normal         p50 {r_p50:>6}  p95 {r_p95:>6}  p99 {r_p99:>6} us   [{tech}]"
    );

    // 2. Contended: strongest priority, but shares contended_core with a spinner.
    //    Sample p99 across several runs so the verdict is not one unlucky draw.
    print!("  contended core @ TimeCritical   p99 over {CONTENDED_RUNS} runs:");
    let mut p99s = Vec::with_capacity(CONTENDED_RUNS);
    let mut starved: Option<String> = None;
    for _ in 0..CONTENDED_RUNS {
        let (tech, fell_back, lat) = run_feeder(contended_core, ThreadPriority::TimeCritical);
        let p99 = pctl(lat, 99);
        print!(" {p99}");
        p99s.push(p99);
        if fell_back {
            starved = Some(tech);
        }
    }
    println!();
    let best = p99s.iter().min().copied().unwrap_or(0);
    let worst = p99s.iter().max().copied().unwrap_or(0);
    println!("                                  best {best} us ... worst {worst} us");
    if let Some(tech) = starved {
        // Only a NAMED-level fallback (-> Normal) makes these numbers a starve
        // rather than the contention tax; a within-level clamp is fine.
        println!(
            "      a run lost TimeCritical entirely: [{tech}] - rtkit budget? see module NOTE"
        );
    }

    stop.store(true, Ordering::SeqCst);
    for h in load {
        let _ = h.join();
    }

    // Computed from THIS run with a 10% noise band - never a hardcoded
    // magnitude. On a contended box the reserved core wins the tail outright; on
    // a many-core box where the TimeCritical feeder already out-prioritizes the
    // spinner, the p99 tails tie (a shared scheduler-tick cost) and placement's
    // real edge is the typical case (p50/p95). The verdict follows the data.
    let margin = r_p99.max(worst) / 10;
    let verdict = if r_p99 + margin < best {
        format!(
            "own core @ Normal wins the tail (reserved p99 {r_p99} us vs contended {best}..{worst} us)"
        )
    } else if best + margin < r_p99 {
        format!(
            "shared core @ TimeCritical wins (contended {best}..{worst} us vs reserved p99 {r_p99} us)"
        )
    } else {
        format!(
            "p99 tails tie (reserved {r_p99} us vs contended {best}..{worst} us); \
             placement's edge here is the typical case - reserved p50/p95 {r_p50}/{r_p95} us"
        )
    };
    println!("\nPlacement vs priority: {verdict}.");
    Ok(())
}
