//! Streaming I/O + Decompression Pipeline Benchmark
//!
//! This benchmark simulates a game asset streaming pipeline:
//! - Thread A reads compressed chunks (simulated I/O)
//! - Thread B decompresses the chunks
//! - Thread C processes the decompressed data (e.g. "renders" it)
//!
//! The benchmark measures end-to-end latency and throughput under
//! different thread scheduling and affinity configurations, with
//! background load to create realistic contention.

use flate2::read::ZlibDecoder;
use gdt_cpus::{CoreType, ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

// Configuration
const CHUNK_SIZE: usize = 64 * 1024; // 64 KB chunks
const CHUNKS_TO_PROCESS: usize = 100;
const BACKGROUND_WORKERS: usize = 4;
const BENCHMARK_DURATION: Duration = Duration::from_secs(15);

// Bounded buffer for pipeline stages with timeout support
struct BoundedBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
    stop: Arc<AtomicBool>,
}

impl<T> BoundedBuffer<T> {
    fn new(capacity: usize, stop: Arc<AtomicBool>) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
            stop,
        }
    }

    fn push(&self, item: T) -> bool {
        let mut buffer = self.buffer.lock().unwrap();

        // Wait until there's space or we're stopping
        while buffer.len() >= self.capacity && !self.stop.load(Ordering::Relaxed) {
            let result = self
                .not_full
                .wait_timeout(buffer, Duration::from_millis(100))
                .unwrap();
            buffer = result.0;

            // Check stop flag periodically
            if self.stop.load(Ordering::Relaxed) {
                return false;
            }
        }

        // Check again after potentially waiting
        if self.stop.load(Ordering::Relaxed) {
            return false;
        }

        buffer.push_back(item);
        self.not_empty.notify_one();
        true
    }

    fn pop(&self) -> Option<T> {
        let mut buffer = self.buffer.lock().unwrap();

        // Wait until there's an item or we're stopping
        while buffer.is_empty() && !self.stop.load(Ordering::Relaxed) {
            let result = self
                .not_empty
                .wait_timeout(buffer, Duration::from_millis(100))
                .unwrap();
            buffer = result.0;

            // Check stop flag periodically
            if self.stop.load(Ordering::Relaxed) && buffer.is_empty() {
                return None;
            }
        }

        if buffer.is_empty() {
            return None;
        }

        let item = buffer.pop_front().unwrap();
        self.not_full.notify_one();
        Some(item)
    }
}

// Asset chunk with metadata
#[derive(Clone)]
struct AssetChunk {
    creation_time: Instant,
    decompress_complete_time: Option<Instant>,
    render_complete_time: Option<Instant>,
    compressed_data: Vec<u8>,
    decompressed_data: Option<Vec<u8>>,
}

// Stats for each stage
struct PipelineStats {
    chunks_processed: usize,
    stage_durations_us: Vec<u64>, // Duration in microseconds for this stage
    total_size_bytes: usize,
}

impl PipelineStats {
    fn new() -> Self {
        Self {
            chunks_processed: 0,
            stage_durations_us: Vec::new(),
            total_size_bytes: 0,
        }
    }
}

// Thread configuration
struct ThreadConfig {
    name: String,
    priority: ThreadPriority,
    pin_to_core: Option<usize>,
}

/// Simulate I/O by compressing random data, then sleeping a bit
fn simulate_io_read(chunk_size: usize) -> AssetChunk {
    let start_time = Instant::now();

    // Generate some random data
    let mut data = Vec::with_capacity(chunk_size);
    for _ in 0..chunk_size {
        data.push(fastrand::u8(..));
    }

    // Compress the data to a reasonable size
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::ZlibEncoder::new(&mut compressed, flate2::Compression::default());
    std::io::copy(&mut data.as_slice(), &mut encoder).unwrap();
    encoder.finish().unwrap();

    // Simulate disk I/O delay (SSD-like latency)
    let io_delay = fastrand::u64(1..5);
    thread::sleep(Duration::from_millis(io_delay));

    AssetChunk {
        creation_time: start_time,
        decompress_complete_time: None,
        render_complete_time: None,
        compressed_data: compressed,
        decompressed_data: None,
    }
}

/// Decompress a chunk
fn decompress_chunk(mut chunk: AssetChunk) -> AssetChunk {
    let mut decompressed = Vec::new();
    let mut decoder = ZlibDecoder::new(chunk.compressed_data.as_slice());
    decoder.read_to_end(&mut decompressed).unwrap();

    chunk.decompressed_data = Some(decompressed);
    chunk.decompress_complete_time = Some(Instant::now());
    chunk
}

/// Simulate rendering/processing the decompressed data
fn process_chunk(mut chunk: AssetChunk) -> AssetChunk {
    let data = chunk.decompressed_data.as_ref().unwrap();

    // Simulate some memory operations on the data
    let mut sum = 0u64;
    for &byte in data.iter().step_by(1024) {
        // Sample every 1KB
        sum = sum.wrapping_add(byte as u64);
    }

    // Artificial delay proportional to data size
    let process_time_us = (data.len() / 1024) as u64; // 1µs per KB
    let start = Instant::now();
    while start.elapsed().as_micros() < process_time_us as u128 {
        std::hint::spin_loop();
    }

    chunk.render_complete_time = Some(Instant::now());
    chunk
}

/// Thread A: Reads chunks
fn reader_thread(
    stop: Arc<AtomicBool>,
    buffer: Arc<BoundedBuffer<AssetChunk>>,
    config: ThreadConfig,
) -> PipelineStats {
    println!("Starting reader thread with {:?}", config);

    // Configure thread
    if let Err(e) = set_thread_priority(config.priority) {
        eprintln!("Failed to set reader thread priority: {}", e);
    }

    if let Some(core_id) = config.pin_to_core {
        if let Err(e) = pin_thread_to_core(core_id) {
            eprintln!("Failed to pin reader thread to core {}: {}", core_id, e);
        }
    }

    let mut stats = PipelineStats::new();
    let mut chunk_id = 0;

    // Read until CHUNKS_TO_PROCESS or until stopped
    while !stop.load(Ordering::Relaxed) && chunk_id < CHUNKS_TO_PROCESS {
        let start = Instant::now();

        let chunk = simulate_io_read(CHUNK_SIZE);
        let size = chunk.compressed_data.len();

        // Push to next stage - break if stopped
        if !buffer.push(chunk) {
            break;
        }

        let duration_us = start.elapsed().as_micros() as u64;
        stats.stage_durations_us.push(duration_us);
        stats.chunks_processed += 1;
        stats.total_size_bytes += size;

        chunk_id += 1;
    }

    // Signal that no more data is coming
    println!("Reader thread finished after {} chunks", chunk_id);
    stats
}

/// Thread B: Decompresses chunks
fn decompressor_thread(
    stop: Arc<AtomicBool>,
    input_buffer: Arc<BoundedBuffer<AssetChunk>>,
    output_buffer: Arc<BoundedBuffer<AssetChunk>>,
    config: ThreadConfig,
) -> PipelineStats {
    println!("Starting decompressor thread with {:?}", config);

    // Configure thread
    if let Err(e) = set_thread_priority(config.priority) {
        eprintln!("Failed to set decompressor thread priority: {}", e);
    }

    if let Some(core_id) = config.pin_to_core {
        if let Err(e) = pin_thread_to_core(core_id) {
            eprintln!(
                "Failed to pin decompressor thread to core {}: {}",
                core_id, e
            );
        }
    }

    let mut stats = PipelineStats::new();

    while !stop.load(Ordering::Relaxed) {
        // Try to get a chunk, with timeout to check stop flag periodically
        let chunk = match input_buffer.pop() {
            Some(chunk) => chunk,
            None => {
                // No more chunks and we've been told to stop
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                // No chunks yet, but keep going
                thread::yield_now();
                continue;
            }
        };

        let start = Instant::now();

        let decompressed = decompress_chunk(chunk);
        let size = decompressed.decompressed_data.as_ref().unwrap().len();

        // Push to next stage - break if stopped
        if !output_buffer.push(decompressed) {
            break;
        }

        let duration_us = start.elapsed().as_micros() as u64;
        stats.stage_durations_us.push(duration_us);
        stats.chunks_processed += 1;
        stats.total_size_bytes += size;
    }

    println!("Decompressor thread finished");
    stats
}

/// Thread C: Processes/renders chunks
fn processor_thread(
    stop: Arc<AtomicBool>,
    input_buffer: Arc<BoundedBuffer<AssetChunk>>,
    completed_chunks: Arc<Mutex<Vec<AssetChunk>>>,
    config: ThreadConfig,
) -> PipelineStats {
    println!("Starting processor thread with {:?}", config);

    // Configure thread
    if let Err(e) = set_thread_priority(config.priority) {
        eprintln!("Failed to set processor thread priority: {}", e);
    }

    if let Some(core_id) = config.pin_to_core {
        if let Err(e) = pin_thread_to_core(core_id) {
            eprintln!("Failed to pin processor thread to core {}: {}", core_id, e);
        }
    }

    let mut stats = PipelineStats::new();

    while !stop.load(Ordering::Relaxed) {
        // Try to get a chunk
        let chunk = match input_buffer.pop() {
            Some(chunk) => chunk,
            None => {
                // No more chunks and we've been told to stop
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                // No chunks yet, but keep going
                thread::yield_now();
                continue;
            }
        };

        let start = Instant::now();

        let processed = process_chunk(chunk);
        let size = processed.decompressed_data.as_ref().unwrap().len();

        // Store completed chunk
        completed_chunks.lock().unwrap().push(processed);

        let duration_us = start.elapsed().as_micros() as u64;
        stats.stage_durations_us.push(duration_us);
        stats.chunks_processed += 1;
        stats.total_size_bytes += size;
    }

    println!("Processor thread finished");
    stats
}

/// Background worker to create CPU load
fn background_worker(stop: Arc<AtomicBool>, id: usize) {
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

    // Simple CPU-bound work
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

/// Calculate stats for a set of durations
fn calculate_stats(durations: &[u64]) -> (u64, u64, u64, u64, f64) {
    if durations.is_empty() {
        return (0, 0, 0, 0, 0.0);
    }

    let mut sorted = durations.to_vec();
    sorted.sort();

    let min = *sorted.first().unwrap();
    let max = *sorted.last().unwrap();

    let p50 = sorted[sorted.len() * 50 / 100];
    let p95 = sorted[sorted.len() * 95 / 100];

    let avg = sorted.iter().sum::<u64>() as f64 / sorted.len() as f64;

    (min, p50, p95, max, avg)
}

/// Runs a complete pipeline benchmark with the given configuration
fn run_pipeline_benchmark(
    reader_config: ThreadConfig,
    decompressor_config: ThreadConfig,
    processor_config: ThreadConfig,
    worker_count: usize,
) -> Vec<AssetChunk> {
    println!("\n--- Running Pipeline Benchmark ---");
    println!("Reader: {:?}", reader_config);
    println!("Decompressor: {:?}", decompressor_config);
    println!("Processor: {:?}", processor_config);
    println!("Background workers: {}", worker_count);

    let stop = Arc::new(AtomicBool::new(false));

    // Create bounded buffers between pipeline stages
    let reader_to_decomp = Arc::new(BoundedBuffer::new(10, stop.clone()));
    let decomp_to_processor = Arc::new(BoundedBuffer::new(10, stop.clone()));
    let completed_chunks = Arc::new(Mutex::new(Vec::new()));

    // Start background workers
    let worker_handles: Vec<_> = (0..worker_count)
        .map(|id| {
            let stop = stop.clone();
            thread::spawn(move || background_worker(stop, id))
        })
        .collect();

    // Start pipeline threads
    let reader_stop = stop.clone();
    let reader_buffer = reader_to_decomp.clone();
    let reader_config_clone = reader_config.clone();

    let reader_handle =
        thread::spawn(move || reader_thread(reader_stop, reader_buffer, reader_config_clone));

    let decomp_stop = stop.clone();
    let decomp_in = reader_to_decomp.clone();
    let decomp_out = decomp_to_processor.clone();
    let decomp_config_clone = decompressor_config.clone();

    let decomp_handle = thread::spawn(move || {
        decompressor_thread(decomp_stop, decomp_in, decomp_out, decomp_config_clone)
    });

    let proc_stop = stop.clone();
    let proc_in = decomp_to_processor.clone();
    let proc_out = completed_chunks.clone();
    let proc_config_clone = processor_config.clone();

    let proc_handle =
        thread::spawn(move || processor_thread(proc_stop, proc_in, proc_out, proc_config_clone));

    // Let the benchmark run
    thread::sleep(BENCHMARK_DURATION);

    // Stop all threads
    println!("Stopping pipeline threads...");
    stop.store(true, Ordering::SeqCst);

    // Wait for pipeline threads to finish
    let reader_res = match reader_handle.join() {
        Ok(res) => res,
        Err(_) => {
            eprintln!("Reader thread panicked!");
            PipelineStats::new()
        }
    };

    let decomp_res = match decomp_handle.join() {
        Ok(res) => res,
        Err(_) => {
            eprintln!("Decompressor thread panicked!");
            PipelineStats::new()
        }
    };

    let proc_res = match proc_handle.join() {
        Ok(res) => res,
        Err(_) => {
            eprintln!("Processor thread panicked!");
            PipelineStats::new()
        }
    };

    // Stop background workers
    println!("Stopping background workers...");
    for handle in worker_handles {
        let _ = handle.join();
    }

    // Analyze results
    let completed = completed_chunks.lock().unwrap().clone();

    println!("\nPipeline Results:");
    println!(
        "  Reader: processed {} chunks, {} bytes",
        reader_res.chunks_processed, reader_res.total_size_bytes
    );
    if !reader_res.stage_durations_us.is_empty() {
        let (r_min, r_p50, r_p95, r_max, r_avg) = calculate_stats(&reader_res.stage_durations_us);
        println!(
            "    Time per chunk (µs): min={}, p50={}, p95={}, max={}, avg={:.2}",
            r_min, r_p50, r_p95, r_max, r_avg
        );
    }

    println!(
        "  Decompressor: processed {} chunks, {} bytes",
        decomp_res.chunks_processed, decomp_res.total_size_bytes
    );
    if !decomp_res.stage_durations_us.is_empty() {
        let (d_min, d_p50, d_p95, d_max, d_avg) = calculate_stats(&decomp_res.stage_durations_us);
        println!(
            "    Time per chunk (µs): min={}, p50={}, p95={}, max={}, avg={:.2}",
            d_min, d_p50, d_p95, d_max, d_avg
        );
    }

    println!(
        "  Processor: processed {} chunks, {} bytes",
        proc_res.chunks_processed, proc_res.total_size_bytes
    );
    if !proc_res.stage_durations_us.is_empty() {
        let (p_min, p_p50, p_p95, p_max, p_avg) = calculate_stats(&proc_res.stage_durations_us);
        println!(
            "    Time per chunk (µs): min={}, p50={}, p95={}, max={}, avg={:.2}",
            p_min, p_p50, p_p95, p_max, p_avg
        );
    }

    // Calculate end-to-end latency
    if !completed.is_empty() {
        let mut e2e_latencies = Vec::new();

        for chunk in &completed {
            if let (start, Some(end)) = (chunk.creation_time, chunk.render_complete_time) {
                let latency = end.duration_since(start).as_micros() as u64;
                e2e_latencies.push(latency);
            }
        }

        if !e2e_latencies.is_empty() {
            let (e_min, e_p50, e_p95, e_max, e_avg) = calculate_stats(&e2e_latencies);
            println!("\nEnd-to-end latency (µs):");
            println!(
                "  min={}, p50={}, p95={}, max={}, avg={:.2}",
                e_min, e_p50, e_p95, e_max, e_avg
            );

            let throughput = completed.len() as f64 / BENCHMARK_DURATION.as_secs_f64();
            println!("\nThroughput: {:.2} chunks/second", throughput);
        }
    }

    completed
}

impl Clone for ThreadConfig {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            priority: self.priority,
            pin_to_core: self.pin_to_core,
        }
    }
}

impl std::fmt::Debug for ThreadConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:?}, pin: {:?})",
            self.name, self.priority, self.pin_to_core
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Streaming I/O + Decompression Pipeline Benchmark");
    println!("================================================");

    // Initialize RNG
    fastrand::seed(0x5678);

    // Get CPU info
    let cpu_info = gdt_cpus::cpu_info()?;

    println!("CPU Info:");
    println!("  Model: {}", cpu_info.model_name);
    println!("  Physical cores: {}", cpu_info.total_physical_cores);
    println!("  Logical cores: {}", cpu_info.total_logical_processors);
    println!("  Performance cores: {}", cpu_info.total_performance_cores);
    println!("  Efficiency cores: {}", cpu_info.total_efficiency_cores);

    // Collect P-cores and E-cores
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

    // Test 1: Baseline - all threads normal priority, no pinning
    let baseline_reader = ThreadConfig {
        name: "Reader".to_string(),
        priority: ThreadPriority::Normal,
        pin_to_core: None,
    };

    let baseline_decomp = ThreadConfig {
        name: "Decompressor".to_string(),
        priority: ThreadPriority::Normal,
        pin_to_core: None,
    };

    let baseline_proc = ThreadConfig {
        name: "Processor".to_string(),
        priority: ThreadPriority::Normal,
        pin_to_core: None,
    };

    run_pipeline_benchmark(
        baseline_reader,
        baseline_decomp,
        baseline_proc,
        BACKGROUND_WORKERS,
    );

    // Test 2: Prioritized pipeline - different priorities, no pinning
    let prio_reader = ThreadConfig {
        name: "Reader".to_string(),
        priority: ThreadPriority::AboveNormal,
        pin_to_core: None,
    };

    let prio_decomp = ThreadConfig {
        name: "Decompressor".to_string(),
        priority: ThreadPriority::Highest,
        pin_to_core: None,
    };

    let prio_proc = ThreadConfig {
        name: "Processor".to_string(),
        priority: ThreadPriority::AboveNormal,
        pin_to_core: None,
    };

    run_pipeline_benchmark(prio_reader, prio_decomp, prio_proc, BACKGROUND_WORKERS);

    // Test 3: Pin critical threads to P-cores
    let pcores_reader = ThreadConfig {
        name: "Reader".to_string(),
        priority: ThreadPriority::AboveNormal,
        pin_to_core: p_cores.first().copied(),
    };

    let pcores_decomp = ThreadConfig {
        name: "Decompressor".to_string(),
        priority: ThreadPriority::Highest,
        pin_to_core: p_cores.get(1).or(p_cores.first()).copied(),
    };

    let pcores_proc = ThreadConfig {
        name: "Processor".to_string(),
        priority: ThreadPriority::AboveNormal,
        pin_to_core: p_cores.get(2).or(p_cores.first()).copied(),
    };

    run_pipeline_benchmark(
        pcores_reader,
        pcores_decomp,
        pcores_proc,
        BACKGROUND_WORKERS,
    );

    Ok(())
}
