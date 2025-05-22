//! Frame Loop Jitter Benchmark
//!
//! This benchmark simulates a game render loop that needs to wake up
//! every 16.67ms (60 FPS) and measures how well it can maintain that
//! cadence under system load.
//!
//! The benchmark:
//! - Creates a "frame thread" that simulates a short workload every frame
//! - Spawns background worker threads to generate CPU load
//! - Measures the jitter (deviation from ideal 16.67ms timing)
//! - Tests different thread priorities and CPU affinity settings

use gdt_cpus::{ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

// Target frame time for 60 FPS
const TARGET_FRAME_TIME: Duration = Duration::from_micros(16670);
// How many frames to measure - 10 seconds worth of frames
const FRAMES_TO_MEASURE: usize = 600;
// Short workload to simulate per-frame processing (100µs)
const FRAME_WORK_MICROS: u64 = 100;
// Number of background worker threads to create
const BACKGROUND_WORKERS: usize = 4;

/// Spin-wait for the target duration to simulate CPU work
fn simulate_work(duration_micros: u64) {
    let start = Instant::now();
    while start.elapsed().as_micros() < duration_micros as u128 {
        std::hint::spin_loop();
    }
}

/// CPU-burn worker to create background load
fn background_worker(stop: Arc<AtomicBool>, id: usize) {
    // For even IDs, use normal priority; for odd, use low priority
    let priority = if id % 2 == 0 {
        ThreadPriority::Normal
    } else {
        ThreadPriority::BelowNormal
    };

    let _ = set_thread_priority(priority);

    println!(
        "Background worker {} started with priority {:?}",
        id, priority
    );

    // Simple calculation to burn CPU cycles
    let mut counter: u64 = 0;
    while !stop.load(Ordering::Relaxed) {
        counter = counter.wrapping_add(1);
        counter = counter.wrapping_mul(0xDEAD_BEEF);
        counter = counter.wrapping_add(counter >> 7);
        if counter & 0x1000 != 0 {
            std::hint::spin_loop();
        }
    }
}

/// Runs the frame loop test with given configuration
fn run_frame_loop_test(
    frame_thread_priority: ThreadPriority,
    pin_to_core: Option<usize>,
    worker_count: usize,
) -> Vec<i64> {
    println!("\n--- Running Frame Loop Test ---");
    println!("Frame thread priority: {:?}", frame_thread_priority);
    println!("Pin to core: {:?}", pin_to_core);
    println!("Background workers: {}", worker_count);

    let stop = Arc::new(AtomicBool::new(false));

    // Spawn background workers to create CPU load
    let worker_handles: Vec<_> = (0..worker_count)
        .map(|id| {
            let stop = stop.clone();
            thread::spawn(move || background_worker(stop, id))
        })
        .collect();

    // Let's collect actual frame time deltas
    let frame_times = Arc::new(std::sync::Mutex::new(Vec::with_capacity(FRAMES_TO_MEASURE)));
    let frame_times_clone = frame_times.clone();

    // Spawn our frame thread
    let frame_handle = thread::spawn(move || {
        // Set priority and pin if requested
        if let Err(e) = set_thread_priority(frame_thread_priority) {
            eprintln!("Failed to set frame thread priority: {}", e);
        }

        if let Some(core_id) = pin_to_core {
            if let Err(e) = pin_thread_to_core(core_id) {
                eprintln!("Failed to pin frame thread to core {}: {}", core_id, e);
            }
        }

        // Prime the thread and let it stabilize
        thread::sleep(Duration::from_millis(100));

        let mut last_frame_time = Instant::now();
        let mut frame_count = 0;

        while frame_count < FRAMES_TO_MEASURE {
            // Wait until next frame target time
            let target_time = last_frame_time + TARGET_FRAME_TIME;
            let now = Instant::now();

            if now < target_time {
                // Sleep for most of the wait, then spin for precision
                let sleep_time = target_time
                    .duration_since(now)
                    .saturating_sub(Duration::from_micros(500));
                if !sleep_time.is_zero() {
                    thread::sleep(sleep_time);
                }

                // Spin-wait for the remainder
                while Instant::now() < target_time {
                    std::hint::spin_loop();
                }
            }

            // Record the actual frame time delta
            let now = Instant::now();
            let frame_delta = now.duration_since(last_frame_time);
            frame_times_clone.lock().unwrap().push(frame_delta);

            // Do some simulated frame work
            simulate_work(FRAME_WORK_MICROS);

            last_frame_time = now;
            frame_count += 1;
        }
    });

    // Wait for frame thread to finish
    frame_handle.join().unwrap();

    // Stop background workers
    stop.store(true, Ordering::SeqCst);
    for handle in worker_handles {
        handle.join().unwrap();
    }

    // Calculate jitter (deviation from ideal frame time)
    let frame_times = frame_times.lock().unwrap();
    frame_times
        .iter()
        .map(|&duration| {
            let micros = duration.as_micros() as i64;
            let target_micros = TARGET_FRAME_TIME.as_micros() as i64;
            micros - target_micros // positive = longer than target, negative = shorter
        })
        .collect()
}

/// Calculate percentiles from a list of samples
fn calculate_percentiles(mut samples: Vec<i64>) -> (i64, i64, i64, i64) {
    samples.sort();
    let len = samples.len();

    let min = *samples.first().unwrap_or(&0);
    let p50 = samples[len * 50 / 100];
    let p95 = samples[len * 95 / 100];
    let p99 = samples[len * 99 / 100];

    (min, p50, p95, p99)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Frame Loop Jitter Benchmark");
    println!("===========================");

    // Get CPU info for making informed decisions
    let cpu_info = gdt_cpus::cpu_info()?;

    println!("CPU Info:");
    println!("  Model: {}", cpu_info.model_name);
    println!("  Performance cores: {}", cpu_info.total_performance_cores);
    println!("  Efficiency cores: {}", cpu_info.total_efficiency_cores);
    println!("  Logical cores: {}", cpu_info.total_logical_processors);

    // Find a performance core to pin to, if any
    let perf_core_id = if cpu_info.total_performance_cores > 0 {
        // Find the first performance core
        let mut perf_core = None;
        for socket in &cpu_info.sockets {
            for core in &socket.cores {
                if core.core_type == gdt_cpus::CoreType::Performance {
                    perf_core = Some(core.logical_processor_ids[0]);
                    break;
                }
            }
            if perf_core.is_some() {
                break;
            }
        }
        perf_core
    } else {
        None
    };

    // Test 1: Normal priority, no pinning
    let jitter1 = run_frame_loop_test(ThreadPriority::Normal, None, BACKGROUND_WORKERS);
    let (min1, p50_1, p95_1, p99_1) = calculate_percentiles(jitter1);

    // Test 2: Above normal priority, no pinning
    let jitter2 = run_frame_loop_test(ThreadPriority::AboveNormal, None, BACKGROUND_WORKERS);
    let (min2, p50_2, p95_2, p99_2) = calculate_percentiles(jitter2);

    // Test 3: High priority, no pinning
    let jitter3 = run_frame_loop_test(ThreadPriority::Highest, None, BACKGROUND_WORKERS);
    let (min3, p50_3, p95_3, p99_3) = calculate_percentiles(jitter3);

    // Test 4: High priority with P-core pinning (if available)
    let jitter4 = run_frame_loop_test(ThreadPriority::Highest, perf_core_id, BACKGROUND_WORKERS);
    let (min4, p50_4, p95_4, p99_4) = calculate_percentiles(jitter4);

    // Test 5: Time critical priority, no pinning
    let jitter5 = run_frame_loop_test(ThreadPriority::TimeCritical, None, BACKGROUND_WORKERS);
    let (min5, p50_5, p95_5, p99_5) = calculate_percentiles(jitter5);

    // Test 6: High priority with P-core pinning (if available)
    let jitter6 = run_frame_loop_test(
        ThreadPriority::TimeCritical,
        perf_core_id,
        BACKGROUND_WORKERS,
    );
    let (min6, p50_6, p95_6, p99_6) = calculate_percentiles(jitter6);

    // Print results summary
    println!("\nResults Summary (jitter in microseconds):");
    println!("Configuration                      |   Min |   p50 |   p95 |   p99");
    println!("-----------------------------------|-------|-------|-------|-------");
    println!(
        "Normal Priority, No Pin            | {:5} | {:5} | {:5} | {:5}",
        min1, p50_1, p95_1, p99_1
    );
    println!(
        "Above Normal Priority, No Pin      | {:5} | {:5} | {:5} | {:5}",
        min2, p50_2, p95_2, p99_2
    );
    println!(
        "Highest Priority, No Pin           | {:5} | {:5} | {:5} | {:5}",
        min3, p50_3, p95_3, p99_3
    );
    println!(
        "Highest Priority, P-core Pin       | {:5} | {:5} | {:5} | {:5}",
        min4, p50_4, p95_4, p99_4
    );
    println!(
        "TimeCritical Priority, No Pin      | {:5} | {:5} | {:5} | {:5}",
        min5, p50_5, p95_5, p99_5
    );
    println!(
        "TimeCritical Priority, P-core Pin  | {:5} | {:5} | {:5} | {:5}",
        min6, p50_6, p95_6, p99_6
    );

    Ok(())
}
