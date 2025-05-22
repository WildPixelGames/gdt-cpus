//! Benchmark audio latency
//!
//! This benchmark measures the audio latency of a system under different
//! thread priorities and CPU load conditions.
//!
//! The benchmark:
//! - Creates a thread that simulates an audio driver spin-wait
//! - Spawns background worker threads to generate CPU load
//! - Measures the jitter (deviation from ideal timing)
//! - Tests different thread priorities and CPU affinity settings

use gdt_cpus::{ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

// Spin-wait hint
#[inline]
fn cpu_relax() {
    std::hint::spin_loop();
}

/// Saturate P-cores
fn spawn_pcore_load(stop: Arc<AtomicBool>, n_pcores: usize) -> Vec<thread::JoinHandle<()>> {
    (0..n_pcores)
        .map(|core| {
            let stop = stop.clone();
            thread::spawn(move || {
                // grant highest priority to load P-core
                let _ = set_thread_priority(ThreadPriority::TimeCritical);
                let _ = pin_thread_to_core(core);
                while !stop.load(Ordering::SeqCst) {
                    cpu_relax();
                }
            })
        })
        .collect()
}

/// Test audio: spin-wait + calculate jitter
fn run_audio_latency_test(
    stop: Arc<AtomicBool>,
    priority: ThreadPriority,
    cores_to_occupy: usize,
) -> Vec<u64> {
    // audio parameters
    const SAMPLE_RATE: u64 = 48_000;
    const BUFFER_SAMPLES: u64 = 256;
    const BUFFER_US: u64 = (BUFFER_SAMPLES * 1_000_000) / SAMPLE_RATE; // ~5333 µs
    const TEST_DURATION_SECS: u64 = 10;
    let expected_buffers = (TEST_DURATION_SECS * 1_000_000) / BUFFER_US;

    // Saturate P-cores
    let pcores = cores_to_occupy;
    let bg_stop = stop.clone();
    let bg_handles = spawn_pcore_load(bg_stop, pcores);

    // Audio driver spin-wait
    let latencies = Arc::new(std::sync::Mutex::new(Vec::with_capacity(
        expected_buffers as usize,
    )));
    let lat_cloned = latencies.clone();
    let stop_clone = stop.clone();

    let handle = thread::spawn(move || {
        // Pin & set priority
        let _ = pin_thread_to_core(0);
        let _ = set_thread_priority(priority);

        let mut last = Instant::now();
        while !stop_clone.load(Ordering::SeqCst)
            && (lat_cloned.lock().unwrap().len() as u64) < expected_buffers
        {
            let target = last + Duration::from_micros(BUFFER_US);
            // Spin-wait till deadline
            loop {
                let now = Instant::now();
                if now >= target {
                    // Record jitter = actual time - deadline
                    let over = now.duration_since(target).as_micros() as u64;
                    lat_cloned.lock().unwrap().push(over);
                    last = now;
                    break;
                }
                cpu_relax();
            }
        }
    });

    handle.join().unwrap();

    // Stop bg load
    stop.store(true, Ordering::SeqCst);
    for h in bg_handles {
        let _ = h.join();
    }

    Arc::try_unwrap(latencies).unwrap().into_inner().unwrap()
}

/// Calculates percentiles 50, 95, 99
fn percentiles(mut data: Vec<u64>) -> (u64, u64, u64) {
    data.sort_unstable();
    let n = data.len();
    let p50 = data[n * 50 / 100];
    let p95 = data[n * 95 / 100];
    let p99 = data[n * 99 / 100];
    (p50, p95, p99)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get CPU information
    let info = gdt_cpus::cpu_info()?;

    println!("CPU Info:");
    println!("  Model: {}", info.model_name);
    println!("  Physical cores: {}", info.total_physical_cores);
    println!("  Logical cores: {}", info.total_logical_processors);
    println!("  Performance cores: {}", info.total_performance_cores);
    println!("  Efficiency cores: {}", info.total_efficiency_cores);

    let stop = Arc::new(AtomicBool::new(false));

    println!(
        "Benchmarking audio latency with {} cores occupied",
        info.total_performance_cores
    );

    for priority in [
        ThreadPriority::Background,
        ThreadPriority::Lowest,
        ThreadPriority::BelowNormal,
        ThreadPriority::Normal,
        ThreadPriority::AboveNormal,
        ThreadPriority::Highest,
        ThreadPriority::TimeCritical,
    ] {
        stop.store(false, Ordering::SeqCst);
        println!(" Benchmarking priority: {:?}", priority);
        let lats = run_audio_latency_test(
            stop.clone(),
            priority,
            info.total_performance_cores as usize,
        );
        let (p50, p95, p99) = percentiles(lats);
        println!("  p50: {}µs, p95: {}µs, p99: {}µs", p50, p95, p99);
    }

    Ok(())
}
