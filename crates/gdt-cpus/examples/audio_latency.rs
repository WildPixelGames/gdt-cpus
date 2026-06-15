//! How much priority does an audio feeder need to hold its cadence under load?
//!
//! Every P-core is saturated by a `Normal`-priority spinner - the engine's
//! job-system worker pool, which is what a real game runs its cores at. A
//! 48 kHz audio feeder is pinned to one of those (now contended) cores and we
//! sweep its priority. `Normal` load is the realistic adversary (the pool is
//! the feeder's true competition) and an honest one: nice 0 is set directly, so
//! all the spinners are identical every run.
//!
//! On Linux the priority ladder is timeshare nice, so the sweep is a CFS weight
//! gradient against nice 0: `Background`/`Lowest`/`BelowNormal` get a sliver and
//! starve, `Normal` ties the pool, and `AboveNormal`/`Highest`/`TimeCritical`
//! outweigh it and own their slice - jitter collapses toward zero. The takeaway:
//! lift the feeder above the pool and the stutter stops.
//!
//! The pool is spawned once and runs the whole sweep; only the feeder is rebuilt
//! per priority. Pass a priority name to benchmark a single rung, e.g.
//! `audio_latency highest`.

mod support;

use gdt_cpus::{CoreKind, ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

// audio feeder parameters (48 kHz, 256-frame buffer)
const SAMPLE_RATE: u64 = 48_000;
const BUFFER_SAMPLES: u64 = 256;
const BUFFER_US: u64 = (BUFFER_SAMPLES * 1_000_000) / SAMPLE_RATE; // ~5333 us
const TEST_DURATION_SECS: u64 = 10;
const EXPECTED_BUFFERS: u64 = (TEST_DURATION_SECS * 1_000_000) / BUFFER_US;
// Wall-clock cap. Completion is buffer-count-based and a starved feeder gathers
// buffers slowly, so without a cap the Background rung runs for minutes; hitting
// the cap is a result (completed/expected), not a failure. Under a Normal load
// the feeder always keeps a CFS sliver, so it reaches this check itself - no
// need to stop the pool to release it.
const HARD_CAP: Duration = Duration::from_secs(TEST_DURATION_SECS + 5);

#[inline]
fn cpu_relax() {
    std::hint::spin_loop();
}

/// One `Normal`-priority spinner per given P-core - the worker pool.
///
/// NOTE: placement uses real LP ids from the topology, never `0..n` index
/// arithmetic - the latter silently lands on E-cores (or holes) on hybrid CPUs.
fn spawn_pcore_load(stop: Arc<AtomicBool>, lp_ids: &[usize]) -> Vec<thread::JoinHandle<()>> {
    lp_ids
        .iter()
        .map(|&lp_id| {
            let stop = stop.clone();
            thread::spawn(move || {
                let _ = set_thread_priority(ThreadPriority::Normal);
                let _ = pin_thread_to_core(lp_id);
                while !stop.load(Ordering::SeqCst) {
                    cpu_relax();
                }
            })
        })
        .collect()
}

/// Measure one priority rung against the running pool: pin a feeder to
/// `audio_core`, set its priority, spin-wait to each 5333 us deadline, record
/// the overshoot. Returns the per-buffer jitter (us).
///
/// The feeder prints its own requested-vs-effective identity the moment its
/// priority is set, so it shows up DURING the ~10 s measurement rather than only
/// after the join - you can watch what is actually running.
///
/// NOTE: the feeder spin-waits instead of sleeping, so the measured jitter is
/// preemption overshoot of a 100%-CPU thread on a contended core, not
/// sleep-wakeup latency.
fn run_audio_feeder(audio_core: usize, priority: ThreadPriority) -> Vec<u64> {
    thread::spawn(move || {
        let _ = pin_thread_to_core(audio_core);
        let priority_result = set_thread_priority(priority);
        println!(
            "  {priority:?} {}",
            support::priority_bracket(priority, &priority_result)
        );

        let mut latencies = Vec::with_capacity(EXPECTED_BUFFERS as usize);
        let hard_deadline = Instant::now() + HARD_CAP;
        let mut last = Instant::now();
        while Instant::now() < hard_deadline && (latencies.len() as u64) < EXPECTED_BUFFERS {
            let target = last + Duration::from_micros(BUFFER_US);
            loop {
                let now = Instant::now();
                if now >= target {
                    latencies.push(now.duration_since(target).as_micros() as u64);
                    last = now;
                    break;
                }
                if now >= hard_deadline {
                    break;
                }
                cpu_relax();
            }
        }
        latencies
    })
    .join()
    .unwrap()
}

/// Calculates percentiles 50, 95, 99
fn percentiles(mut data: Vec<u64>) -> (u64, u64, u64) {
    if data.is_empty() {
        return (0, 0, 0);
    }
    data.sort_unstable();
    let n = data.len();
    let p50 = data[n * 50 / 100];
    let p95 = data[n * 95 / 100];
    let p99 = data[n * 99 / 100];
    (p50, p95, p99)
}

/// Human-readable duration. Sub-millisecond stays in us - the precision matters
/// for a healthy feeder; >=1 ms switches to ms (>=1 s to s) because us digits on a
/// starved feeder are just noise. Renders <=7 chars for column alignment.
fn fmt_dur(us: u64) -> String {
    if us >= 1_000_000 {
        format!("{:.1}s", us as f64 / 1_000_000.0)
    } else if us >= 1_000 {
        format!("{:.1}ms", us as f64 / 1_000.0)
    } else {
        format!("{us}us")
    }
}

/// Single-rung takeaway: report ONLY what this rung measured. It makes no claim
/// about rungs that did not run.
fn single_rung_takeaway(priority: ThreadPriority, completed: u64) -> String {
    format!("Single rung {priority:?}: {completed}/{EXPECTED_BUFFERS} buffers delivered.")
}

/// Full-sweep takeaway, COMPUTED from the per-rung delivery counts (priorities
/// are swept weakest-first). It names the threshold rung the run actually showed
/// instead of asserting a hardcoded "Below Normal starves" the box can refute.
fn full_sweep_takeaway(results: &[(ThreadPriority, u64)]) -> String {
    match results.iter().find(|(_, c)| *c >= EXPECTED_BUFFERS) {
        None => {
            "No priority kept the feeder fed on this box: every rung missed buffers.".to_string()
        }
        Some(&(p, _)) if p == results[0].0 => format!(
            "Even {p:?} delivered every buffer here - this box isn't contended enough to starve the feeder."
        ),
        Some(&(p, _)) => {
            format!("{p:?} and up keep up (every buffer delivered); weaker levels starved.")
        }
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;

    #[test]
    fn single_rung_reports_only_what_it_measured() {
        assert_eq!(
            single_rung_takeaway(ThreadPriority::Highest, EXPECTED_BUFFERS),
            format!(
                "Single rung Highest: {EXPECTED_BUFFERS}/{EXPECTED_BUFFERS} buffers delivered."
            )
        );
    }

    #[test]
    fn full_sweep_names_the_threshold_rung_from_the_data() {
        let results = [
            (ThreadPriority::Background, 0),
            (ThreadPriority::Lowest, EXPECTED_BUFFERS / 2),
            (ThreadPriority::Normal, EXPECTED_BUFFERS),
            (ThreadPriority::Highest, EXPECTED_BUFFERS),
        ];
        assert_eq!(
            full_sweep_takeaway(&results),
            "Normal and up keep up (every buffer delivered); weaker levels starved."
        );
    }

    #[test]
    fn full_sweep_when_nothing_starves() {
        let results = [
            (ThreadPriority::Background, EXPECTED_BUFFERS),
            (ThreadPriority::Normal, EXPECTED_BUFFERS),
        ];
        assert_eq!(
            full_sweep_takeaway(&results),
            "Even Background delivered every buffer here - this box isn't contended enough to starve the feeder."
        );
    }

    #[test]
    fn full_sweep_when_everything_starves() {
        let results = [(ThreadPriority::Normal, 0), (ThreadPriority::Highest, 1)];
        assert_eq!(
            full_sweep_takeaway(&results),
            "No priority kept the feeder fed on this box: every rung missed buffers."
        );
    }
}

/// Parses a priority name (case- and separator-insensitive, plus short
/// aliases) so a single rung can be benchmarked without waiting out the sweep:
/// `audio_latency abovenormal` (or `above`, `tc`, ...).
fn parse_priority(s: &str) -> Option<ThreadPriority> {
    let norm: String = s
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    Some(match norm.as_str() {
        "background" | "bg" => ThreadPriority::Background,
        "lowest" | "low" => ThreadPriority::Lowest,
        "belownormal" | "below" => ThreadPriority::BelowNormal,
        "normal" | "norm" => ThreadPriority::Normal,
        "abovenormal" | "above" => ThreadPriority::AboveNormal,
        "highest" | "high" => ThreadPriority::Highest,
        "timecritical" | "timecrit" | "critical" | "tc" => ThreadPriority::TimeCritical,
        _ => return None,
    })
}

const ALL_PRIORITIES: [ThreadPriority; 7] = [
    ThreadPriority::Background,
    ThreadPriority::Lowest,
    ThreadPriority::BelowNormal,
    ThreadPriority::Normal,
    ThreadPriority::AboveNormal,
    ThreadPriority::Highest,
    ThreadPriority::TimeCritical,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional arg: benchmark a single priority instead of the full sweep -
    // pairs well with `prlimit --nice=N` for testing one rung's grant path.
    let priorities: Vec<ThreadPriority> = match std::env::args().nth(1) {
        Some(arg) => match parse_priority(&arg) {
            Some(p) => vec![p],
            None => {
                eprintln!(
                    "unknown priority '{arg}'; valid: \
                     background|lowest|belownormal|normal|abovenormal|highest|timecritical"
                );
                std::process::exit(2);
            }
        },
        None => ALL_PRIORITIES.to_vec(),
    };

    let info = gdt_cpus::CpuInfo::detect()?;
    println!(
        "CPU: {} - {} cores / {} threads ({} P + {} E)",
        info.model_name,
        info.num_physical_cores(),
        info.num_logical_cores(),
        info.num_performance_cores(),
        info.num_efficiency_cores()
    );

    // How many of the 7 rungs resolve to a DIFFERENT scheduler weight on this
    // box? On a locked-down machine the top levels collapse onto Normal, and the
    // sweep below then shows them tying Normal's numbers.
    let caps = gdt_cpus::priority_capabilities();
    print!(
        "Distinct priority levels here: {} of 7",
        caps.distinct_levels()
    );
    if caps.distinct(ThreadPriority::Highest, ThreadPriority::Normal) {
        println!(" (each rung maps to a different scheduler weight)");
    } else {
        println!(" - top levels collapse onto Normal (no privilege / no rtkit)");
    }

    // One primary thread (smt_index == 0) per Performance core, by OS LP id.
    let p_lps: Vec<usize> = info
        .lps
        .iter()
        .filter(|lp| lp.smt_index == 0 && lp.kind == CoreKind::Performance)
        .map(|lp| lp.os_id as usize)
        .collect();

    // Spawn the worker pool ONCE - Normal spinners on every P-core, alive for
    // the whole sweep. The audio feeder shares the first one (worst-case
    // contention); only it is rebuilt per priority.
    let stop = Arc::new(AtomicBool::new(false));
    let load = spawn_pcore_load(stop.clone(), &p_lps);
    let audio_core = p_lps[0];

    println!(
        "Synthetic load: {} spinners at Normal priority pinned to {} P-cores \
         (the feeder shares core {}).",
        p_lps.len(),
        p_lps.len(),
        audio_core
    );
    println!("\nRunning audio feeder ~10s (cap 15s) at each priority:\n");

    let single_rung = priorities.len() == 1;
    let mut results: Vec<(ThreadPriority, u64)> = Vec::new();
    for priority in priorities {
        let lats = run_audio_feeder(audio_core, priority);
        let completed = lats.len() as u64;
        results.push((priority, completed));
        let (p50, p95, p99) = percentiles(lats);
        let verdict = if completed < EXPECTED_BUFFERS {
            "  STARVED"
        } else {
            ""
        };
        println!(
            "    p50 {:>7}  p95 {:>7}  p99 {:>7}   {:>4}/{} buffers{}",
            fmt_dur(p50),
            fmt_dur(p95),
            fmt_dur(p99),
            completed,
            EXPECTED_BUFFERS,
            verdict
        );
    }

    // Tear down the pool now that the sweep is done.
    stop.store(true, Ordering::SeqCst);
    for h in load {
        let _ = h.join();
    }

    let takeaway = if single_rung {
        single_rung_takeaway(results[0].0, results[0].1)
    } else {
        full_sweep_takeaway(&results)
    };
    println!("\n{takeaway}");
    Ok(())
}
