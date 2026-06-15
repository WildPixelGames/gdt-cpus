//! Frame pacing under load: can the RENDER THREAD hold 60 FPS while the engine's
//! worker pool saturates the machine - and does it matter how the pool is sized?
//!
//! Each frame the render thread does a FIXED amount of CPU work (calibrated to a
//! few ms on an idle core) and must finish inside the 16.67 ms budget; a frame
//! whose work overruns the budget is a dropped frame. We sweep the render
//! thread's priority against two worker-pool sizes:
//!
//!   Round 1 - pool sized to LOGICAL cores (one worker per hardware thread, the
//!     naive default). Every LP is oversubscribed, so the render thread is forced
//!     to time-slice with a worker and priority becomes load-bearing: it drops
//!     many frames at Normal and none at TimeCritical.
//!   Round 2 - pool sized to PHYSICAL cores (one worker per core, leaving the SMT
//!     siblings free - what a topology-aware engine does). The render thread lands
//!     on a free sibling with its own scheduling slot and keeps its cadence at
//!     ANY priority. Skipped when logical == physical: there is no second pool.
//!
//! The lesson is the contrast: size the pool with `num_physical_cores()` and the
//! render thread has headroom; oversubscribe to `num_logical_cores()` and you
//! manufacture a problem that priority can paper over but shouldn't have to.
//!
//! Work-per-frame is a CLI arg (`frame_jitter <work_ms>`) because the contention
//! cliff moves with all-core clock throttling and SMT contention.

mod support;

use gdt_cpus::{AppliedPriority, ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::hint::black_box;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const FRAME_BUDGET: Duration = Duration::from_micros(16_670); // 60 FPS
// Per-frame CPU work, measured on an idle core. The default is intentionally
// close to the frame budget once SMT contention stretches it; override with
// `frame_jitter <work_ms>` to find the cliff on a different machine.
const DEFAULT_WORK_MS: f64 = 5.5;
const FRAMES: usize = 600;

const LADDER: [ThreadPriority; 4] = [
    ThreadPriority::Normal,
    ThreadPriority::AboveNormal,
    ThreadPriority::Highest,
    ThreadPriority::TimeCritical,
];

/// Pure ALU busywork. A fixed iteration count takes a fixed time on an idle
/// core but STRETCHES under contention - unlike a clock-watching spin, which
/// reads ~constant wall time regardless of how often it was preempted. That
/// stretch is exactly what turns into a dropped frame.
#[inline]
fn work(iters: u64) -> u64 {
    let mut x = 0u64;
    for i in 0..iters {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(i | 1);
    }
    x
}

/// Iteration count that takes ~`target` on THIS idle core, so per-frame work is
/// the same wall time on any machine before contention is added.
fn calibrate(target: Duration) -> u64 {
    let probe: u64 = 20_000_000;
    let t = Instant::now();
    black_box(work(probe));
    let per_iter_ns = (t.elapsed().as_nanos() as f64 / probe as f64).max(0.001);
    (target.as_nanos() as f64 / per_iter_ns) as u64
}

/// CPU-burn background worker pinned to `lp`. Half Normal, half BelowNormal - a
/// realistic non-uniform pool rather than identical competitors.
fn background_worker(stop: Arc<AtomicBool>, id: usize, lp: usize) {
    let _ = pin_thread_to_core(lp);
    let _ = set_thread_priority(if id.is_multiple_of(2) {
        ThreadPriority::Normal
    } else {
        ThreadPriority::BelowNormal
    });
    let mut x = 0u64;
    while !stop.load(Ordering::Relaxed) {
        x = black_box(work(4096));
    }
    black_box(x);
}

/// Run the render thread at `priority` against one worker pinned to each LP in
/// `worker_lps`. The render thread itself is UNPINNED - where it lands (a shared
/// LP under an oversubscribed pool, or a free SMT sibling under a physical-sized
/// one) is the whole point. Returns priority result + frame timing samples.
fn run_frame_loop(
    priority: ThreadPriority,
    worker_lps: &[usize],
    work_iters: u64,
) -> (gdt_cpus::Result<AppliedPriority>, usize, Duration, Duration) {
    let stop = Arc::new(AtomicBool::new(false));
    let handles: Vec<_> = worker_lps
        .iter()
        .enumerate()
        .map(|(id, &lp)| {
            let stop = stop.clone();
            thread::spawn(move || background_worker(stop, id, lp))
        })
        .collect();

    let result = thread::spawn(move || {
        let priority_result = set_thread_priority(priority);
        thread::sleep(Duration::from_millis(100)); // let the pool ramp

        let mut dropped = 0usize;
        let mut worst = Duration::ZERO;
        let mut samples = Vec::with_capacity(FRAMES);
        for _ in 0..FRAMES {
            let start = Instant::now();
            black_box(work(work_iters));
            let wall = start.elapsed();
            samples.push(wall);
            if wall > FRAME_BUDGET {
                dropped += 1;
            }
            worst = worst.max(wall);
            // Vsync wait: finished early -> yield the core to the pool until the
            // next boundary; already over budget -> no slack, start the next frame.
            if let Some(slack) = FRAME_BUDGET.checked_sub(wall) {
                thread::sleep(slack);
            }
        }
        samples.sort_unstable();
        let p99 = samples[samples.len() * 99 / 100];
        (priority_result, dropped, p99, worst)
    })
    .join()
    .unwrap();

    stop.store(true, Ordering::SeqCst);
    for h in handles {
        let _ = h.join();
    }
    result
}

/// Sweep the render thread's priority against one worker pool. Returns the
/// dropped-frame count at Normal priority (the headline number for the contrast).
fn run_round(title: &str, worker_lps: &[usize], work_iters: u64) -> usize {
    println!("{title}");
    let mut normal_drops = 0;
    for priority in LADDER {
        let (priority_result, dropped, p99, worst) =
            run_frame_loop(priority, worker_lps, work_iters);
        if matches!(priority, ThreadPriority::Normal) {
            normal_drops = dropped;
        }
        println!(
            "  render @ {:<13} {:<31} dropped {:>4}/{}   p99 {:>6.1}ms   worst {:>6.1}ms",
            format!("{priority:?}"),
            support::priority_bracket(priority, &priority_result),
            dropped,
            FRAMES,
            p99.as_secs_f64() * 1000.0,
            worst.as_secs_f64() * 1000.0
        );
    }
    normal_drops
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = gdt_cpus::CpuInfo::detect()?;
    println!(
        "CPU: {} - {} P + {} E cores / {} threads",
        info.model_name,
        info.num_performance_cores(),
        info.num_efficiency_cores(),
        info.num_logical_cores()
    );

    let all_lps: Vec<usize> = info.lps.iter().map(|lp| lp.os_id as usize).collect();
    let phys_lps: Vec<usize> = info
        .lps
        .iter()
        .filter(|lp| lp.smt_index == 0)
        .map(|lp| lp.os_id as usize)
        .collect();

    if phys_lps.len() == all_lps.len() {
        println!(
            "frame_jitter: skipped; no SMT detected (logical == physical). \
             Run this on an SMT-capable CPU to compare logical vs physical worker-pool sizing."
        );
        return Ok(());
    }

    let work_ms: f64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_WORK_MS);
    let work_iters = calibrate(Duration::from_micros((work_ms * 1000.0) as u64));
    println!(
        "Render thread: 60 FPS, {:.1}ms work/frame, {} frames. \
         Dropped = work overran the {:.1}ms budget.\n",
        work_ms,
        FRAMES,
        FRAME_BUDGET.as_secs_f64() * 1000.0
    );

    let logical_drops = run_round(
        &format!(
            "Round 1 - worker pool = {} LOGICAL cores (one per hardware thread, oversubscribed):",
            all_lps.len()
        ),
        &all_lps,
        work_iters,
    );

    println!();
    let physical_drops = run_round(
        &format!(
            "Round 2 - worker pool = {} PHYSICAL cores (SMT siblings left free):",
            phys_lps.len()
        ),
        &phys_lps,
        work_iters,
    );
    println!(
        "\nRender frames dropped at Normal: {}/{} (logical pool) vs {}/{} (physical pool).",
        logical_drops, FRAMES, physical_drops, FRAMES
    );

    Ok(())
}
