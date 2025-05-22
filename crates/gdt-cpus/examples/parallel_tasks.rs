//! Multi-Threaded Physics/AI Sweep Benchmark
//!
//! This benchmark simulates a typical game workload where many small
//! independent tasks are processed in parallel (like collision checks,
//! AI computations, physics updates).
//!
//! The benchmark:
//! - Creates a pool of worker threads
//! - Generates a queue of simulated "game jobs" (e.g. collision checks)
//! - Measures throughput (tasks/second) and latency (per-task time)
//! - Tests different thread priorities, affinities, and core types

use gdt_cpus::{CoreType, ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::collections::VecDeque;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

// Configuration
const TOTAL_TASKS: usize = 10_000;
const TASK_COMPLEXITY_MIN: u64 = 10; // Microseconds
const TASK_COMPLEXITY_MAX: u64 = 1000; // Microseconds
const MEASUREMENT_TIME: Duration = Duration::from_secs(5);

// Task simulation
struct Task {
    complexity: u64, // in microseconds
}

// Thread pool configuration
struct WorkerConfig {
    priority: ThreadPriority,
    pin_to_core: Option<usize>,
    name: String,
}

// Results from a worker thread
struct WorkerStats {
    tasks_completed: usize,
    total_processing_time_us: u64,
    per_task_times_us: Vec<u64>,
}

// Queue of tasks
struct TaskQueue {
    tasks: Mutex<VecDeque<Task>>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
        }
    }

    fn generate_tasks(&self, count: usize) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.clear();

        for _ in 0..count {
            // Random complexity between min and max
            let complexity = fastrand::u64(TASK_COMPLEXITY_MIN..=TASK_COMPLEXITY_MAX);
            tasks.push_back(Task { complexity });
        }
    }

    fn take_task(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop_front()
    }
}

/// Simulates executing a task with given complexity (in microseconds)
fn execute_task(complexity_us: u64) {
    // Simulating task execution with spin-wait
    let start = Instant::now();

    // Do some actual work to avoid compiler optimizations
    let mut sum = 0u64;
    while start.elapsed().as_micros() < complexity_us as u128 {
        // Some meaningless work
        sum = sum.wrapping_add(1);
        sum = sum.wrapping_mul(0xDEAD_BEEF);
        sum = sum.wrapping_add(sum >> 7);

        std::hint::spin_loop();
    }
}

/// Worker thread function
fn worker_thread(
    id: usize,
    queue: Arc<TaskQueue>,
    stop: Arc<AtomicBool>,
    config: WorkerConfig,
) -> WorkerStats {
    println!(
        "Worker {} starting: {:?} priority, pin to core: {:?}",
        id, config.priority, config.pin_to_core
    );

    // Configure thread
    if let Err(e) = set_thread_priority(config.priority) {
        eprintln!("Failed to set thread priority: {}", e);
    }

    if let Some(core_id) = config.pin_to_core {
        if let Err(e) = pin_thread_to_core(core_id) {
            eprintln!("Failed to pin thread to core {}: {}", core_id, e);
        }
    }

    // Stats collection
    let mut stats = WorkerStats {
        tasks_completed: 0,
        total_processing_time_us: 0,
        per_task_times_us: Vec::new(),
    };

    // Process tasks until stopped
    while !stop.load(Ordering::Relaxed) {
        if let Some(task) = queue.take_task() {
            let start_time = Instant::now();

            execute_task(task.complexity);

            let elapsed_us = start_time.elapsed().as_micros() as u64;
            stats.tasks_completed += 1;
            stats.total_processing_time_us += elapsed_us;
            stats.per_task_times_us.push(elapsed_us);
        } else {
            // No tasks available, yield to other threads
            thread::yield_now();
        }
    }

    stats
}

/// Calculate statistics from task times
fn calculate_task_stats(task_times: &[u64]) -> (u64, u64, u64, u64) {
    if task_times.is_empty() {
        return (0, 0, 0, 0);
    }

    let mut sorted_times = task_times.to_vec();
    sorted_times.sort();

    let len = sorted_times.len();
    let min = *sorted_times.first().unwrap();
    let max = *sorted_times.last().unwrap();
    let p50 = sorted_times[len * 50 / 100];
    let p99 = sorted_times[len * 99 / 100];

    (min, p50, p99, max)
}

/// Run a benchmark with the given configuration
fn run_benchmark(
    num_workers: usize,
    worker_configs: Vec<WorkerConfig>,
    task_count: usize,
) -> Vec<WorkerStats> {
    println!("\n--- Running benchmark with {} workers ---", num_workers);

    // Create task queue
    let queue = Arc::new(TaskQueue::new());
    queue.generate_tasks(task_count);

    // Create stop flag
    let stop = Arc::new(AtomicBool::new(false));

    // Create workers
    let mut handles = Vec::with_capacity(num_workers);

    for i in 0..num_workers {
        let queue = queue.clone();
        let stop = stop.clone();
        let config = worker_configs[i % worker_configs.len()].clone();

        let handle = thread::spawn(move || worker_thread(i, queue, stop, config));
        handles.push(handle);
    }

    // Let the benchmark run for the specified time
    thread::sleep(MEASUREMENT_TIME);

    // Stop workers
    stop.store(true, Ordering::SeqCst);

    // Collect results
    let mut results = Vec::with_capacity(num_workers);
    for handle in handles {
        results.push(handle.join().unwrap());
    }

    results
}

impl Clone for WorkerConfig {
    fn clone(&self) -> Self {
        Self {
            priority: self.priority,
            pin_to_core: self.pin_to_core,
            name: self.name.clone(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-Threaded Physics/AI Sweep Benchmark");
    println!("=========================================");

    // Initialize a simple RNG
    fastrand::seed(0x1234);

    // Get CPU info
    let cpu_info = gdt_cpus::cpu_info()?;

    println!("CPU Info:");
    println!("  Model: {}", cpu_info.model_name);
    println!("  Physical cores: {}", cpu_info.total_physical_cores);
    println!("  Performance cores: {}", cpu_info.total_performance_cores);
    println!("  Efficiency cores: {}", cpu_info.total_efficiency_cores);
    println!("  Logical cores: {}", cpu_info.total_logical_processors);

    // Collect P-cores and E-cores for pinning
    let mut p_cores = Vec::new();
    let mut e_cores = Vec::new();

    for socket in &cpu_info.sockets {
        for core in &socket.cores {
            if core.logical_processor_ids.is_empty() {
                continue;
            }

            let lp_id = core.logical_processor_ids[0];
            match core.core_type {
                CoreType::Performance => p_cores.push(lp_id),
                CoreType::Efficiency => e_cores.push(lp_id),
                CoreType::Unknown => {}
            }
        }
    }

    println!(
        "Found {} P-cores and {} E-cores",
        p_cores.len(),
        e_cores.len()
    );

    // Determine number of worker threads based on available cores
    let num_workers = cpu_info.total_logical_processors;

    // Test 1: All workers with normal priority, no pinning
    let config1 = vec![WorkerConfig {
        priority: ThreadPriority::Normal,
        pin_to_core: None,
        name: "Normal".to_string(),
    }];

    let results1 = run_benchmark(num_workers, config1, TOTAL_TASKS);

    // Test 2: Mixed worker priorities, no pinning
    let mut config2 = Vec::new();
    config2.push(WorkerConfig {
        priority: ThreadPriority::AboveNormal,
        pin_to_core: None,
        name: "AboveNormal".to_string(),
    });
    config2.push(WorkerConfig {
        priority: ThreadPriority::Normal,
        pin_to_core: None,
        name: "Normal".to_string(),
    });

    let results2 = run_benchmark(num_workers, config2, TOTAL_TASKS);

    // Test 3: If we have both P-cores and E-cores, pin accordingly
    let has_hybrid_arch = !p_cores.is_empty() && !e_cores.is_empty();
    let mut results3 = Vec::new();

    if has_hybrid_arch {
        let mut config3 = Vec::new();

        // Create configs for P-cores with high priority
        for &core_id in &p_cores {
            config3.push(WorkerConfig {
                priority: ThreadPriority::AboveNormal,
                pin_to_core: Some(core_id),
                name: format!("P-core {}", core_id),
            });
        }

        // Create configs for E-cores with normal priority
        for &core_id in &e_cores {
            config3.push(WorkerConfig {
                priority: ThreadPriority::BelowNormal,
                pin_to_core: Some(core_id),
                name: format!("E-core {}", core_id),
            });
        }

        results3 = run_benchmark(num_workers, config3, TOTAL_TASKS);
    }

    // Print aggregate results
    println!("\n--- Results Summary ---");

    // Test 1 results
    println!("\nTest 1: All workers with Normal priority, no pinning");
    print_results_summary(&results1);

    // Test 2 results
    println!("\nTest 2: Mixed worker priorities, no pinning");
    print_results_summary(&results2);

    // Test 3 results (if applicable)
    if has_hybrid_arch {
        println!("\nTest 3: P-cores and E-cores, pinned with different priorities");
        print_results_summary(&results3);
    }

    Ok(())
}

fn print_results_summary(results: &[WorkerStats]) {
    // Calculate total stats
    let total_tasks: usize = results.iter().map(|s| s.tasks_completed).sum();
    let mut all_task_times = Vec::new();

    for result in results {
        all_task_times.extend(&result.per_task_times_us);
    }

    let (min, p50, p99, max) = calculate_task_stats(&all_task_times);

    println!("Total tasks processed: {}", total_tasks);
    println!(
        "Tasks per second: {:.2}",
        total_tasks as f64 / MEASUREMENT_TIME.as_secs_f64()
    );
    println!("Task latency (microseconds):");
    println!("  Min: {}", min);
    println!("  p50: {}", p50);
    println!("  p99: {}", p99);
    println!("  Max: {}", max);

    // Per-worker stats
    println!("\nPer-worker statistics:");
    for (i, result) in results.iter().enumerate() {
        let (w_min, w_p50, w_p99, w_max) = calculate_task_stats(&result.per_task_times_us);
        println!(
            "Worker {}: {} tasks, latency min/p50/p99/max: {}/{}/{}/{} µs",
            i, result.tasks_completed, w_min, w_p50, w_p99, w_max
        );
    }
}
