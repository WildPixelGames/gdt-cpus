//! Background work budget vs frame pacing.
//!
//! The render thread touches a frame-sized working set at 60 FPS while an asset
//! preparation pool inflates large cold assets, copies them into an upload
//! buffer, and touches the result. One primary Performance-core LP is reserved
//! for the render thread; workers are pinned to the remaining primary
//! Performance LPs.
//!
//! This is not an IO streaming model: there is no disk, cancellation, visibility
//! demand, or queue backpressure. It models CPU-heavy runtime background work
//! competing with a frame. The useful engine decision is the frame-safe worker
//! budget: the fastest measured width that stays under the frame budget with
//! headroom. Raw GB/s is supporting data, not the goal.
//!
//! Args: background_budget [work_ms] [asset_mib >= 1] [trials 1..5].

mod support;

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
#[cfg(test)]
use gdt_cpus::Lp;
use gdt_cpus::{CoreKind, ThreadPriority, pin_thread_to_core, set_thread_priority};
use std::hint::black_box;
use std::io::{Read, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const FRAME_BUDGET: Duration = Duration::from_micros(16_670);
const DEFAULT_WORK_MS: f64 = 1.5;
const DEFAULT_ASSET_MIB: usize = 64;
const DEFAULT_TRIALS: usize = 3;
const MAX_TRIALS: usize = 5;
const FRAMES: usize = 120;
const RENDER_BYTES: usize = 16 * 1024 * 1024;
const MIN_ASSETS: usize = 8;
const MAX_ASSETS: usize = 32;
const FRAME_SAFE_P99_MS: f64 = 15.0;
const KNEE_GAIN_FRACTION: f64 = 0.20;
const PRIORITY_SWEEP: [ThreadPriority; 4] = [
    ThreadPriority::Normal,
    ThreadPriority::BelowNormal,
    ThreadPriority::Lowest,
    ThreadPriority::Background,
];

#[derive(Clone, Copy)]
struct WidthPoint {
    workers: usize,
    gbps: f64,
    p99_ms: f64,
    drops: usize,
}

#[derive(Clone, Copy)]
struct CandidateLp {
    os_id: usize,
    l3_domain: u8,
}

#[derive(Clone)]
struct AssetBank {
    assets: Arc<Vec<Arc<Vec<u8>>>>,
    asset_bytes: usize,
}

struct RunContext<'a> {
    bank: &'a AssetBank,
    render_lp: usize,
    worker_lps: &'a [usize],
    work_iters: u64,
}

#[inline]
fn work(iters: u64) -> u64 {
    let mut x = 0u64;
    for i in 0..iters {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(i | 1);
    }
    x
}

fn calibrate(target: Duration) -> u64 {
    let probe: u64 = 20_000_000;
    let t = Instant::now();
    black_box(work(probe));
    let per_iter_ns = (t.elapsed().as_nanos() as f64 / probe as f64).max(0.001);
    (target.as_nanos() as f64 / per_iter_ns) as u64
}

fn worker_width_schedule(physical_workers: usize) -> Vec<usize> {
    let cap = physical_workers.max(1);
    let mut out = Vec::new();
    let mut width = 1;
    while width < cap {
        out.push(width);
        width *= 2;
    }
    out.push(cap);
    out
}

fn choose_bandwidth_knee(points: &[WidthPoint]) -> usize {
    if points.len() <= 1 {
        return 0;
    }
    for i in 0..points.len() - 1 {
        let current = points[i];
        let next = points[i + 1];
        let gain = if current.gbps > 0.0 {
            (next.gbps - current.gbps) / current.gbps
        } else {
            1.0
        };
        if gain <= KNEE_GAIN_FRACTION {
            return i;
        }
    }
    points.len() - 1
}

fn choose_frame_safe_pick(points: &[WidthPoint]) -> usize {
    let mut best_safe: Option<usize> = None;
    for (i, point) in points.iter().enumerate() {
        if point.drops != 0 || point.p99_ms > FRAME_SAFE_P99_MS {
            continue;
        }
        match best_safe {
            Some(best) if point.gbps <= points[best].gbps => {}
            _ => best_safe = Some(i),
        }
    }
    if let Some(best) = best_safe {
        return best;
    }

    let mut best = 0;
    for (i, point) in points.iter().enumerate().skip(1) {
        let current = points[best];
        if point.drops < current.drops
            || (point.drops == current.drops && point.p99_ms < current.p99_ms)
        {
            best = i;
        }
    }
    best
}

fn median_f64(mut values: Vec<f64>) -> f64 {
    values.sort_unstable_by(f64::total_cmp);
    values[values.len() / 2]
}

fn median_usize(mut values: Vec<usize>) -> usize {
    values.sort_unstable();
    values[values.len() / 2]
}

fn median_point(workers: usize, trials: &[WidthPoint]) -> WidthPoint {
    WidthPoint {
        workers,
        gbps: median_f64(trials.iter().map(|p| p.gbps).collect()),
        p99_ms: median_f64(trials.iter().map(|p| p.p99_ms).collect()),
        drops: median_usize(trials.iter().map(|p| p.drops).collect()),
    }
}

fn round_robin_l3_lps(candidates: &[CandidateLp]) -> Vec<usize> {
    let mut domains = Vec::new();
    for candidate in candidates {
        if !domains.contains(&candidate.l3_domain) {
            domains.push(candidate.l3_domain);
        }
    }

    let mut out = Vec::with_capacity(candidates.len());
    let mut pass = 0;
    while out.len() < candidates.len() {
        for &domain in &domains {
            if let Some(candidate) = candidates
                .iter()
                .filter(|c| c.l3_domain == domain)
                .nth(pass)
            {
                out.push(candidate.os_id);
            }
        }
        pass += 1;
    }
    out
}

fn fill_raw_asset(asset_index: usize, raw: &mut [u8]) {
    for (i, b) in raw.iter_mut().enumerate() {
        let stripe = (((i / 64) + asset_index * 17) % 251) as u8;
        let marker = if (i + asset_index * 131).is_multiple_of(4099) {
            (asset_index % 251) as u8
        } else {
            0
        };
        *b = stripe ^ marker;
    }
}

fn build_asset_bank(count: usize, asset_bytes: usize) -> std::io::Result<AssetBank> {
    let mut raw = vec![0u8; asset_bytes];
    let mut assets = Vec::with_capacity(count);
    for i in 0..count {
        fill_raw_asset(i, &mut raw);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw)?;
        assets.push(Arc::new(encoder.finish()?));
        if (i + 1).is_multiple_of(4) || i + 1 == count {
            print!("\rbuilding asset bank: {}/{}", i + 1, count);
            let _ = std::io::stdout().flush();
        }
    }
    println!();
    Ok(AssetBank {
        assets: Arc::new(assets),
        asset_bytes,
    })
}

fn decode_upload_and_touch(asset: &[u8], outbuf: &mut Vec<u8>, upload: &mut [u8]) -> usize {
    outbuf.clear();
    let mut decoder = ZlibDecoder::new(asset);
    if decoder.read_to_end(outbuf).is_err() {
        return 0;
    }

    let mut sum = 0u64;
    for (&b, u) in outbuf.iter().zip(upload.iter_mut()) {
        let v = b ^ (sum as u8);
        *u = v;
        sum = sum.wrapping_add(v as u64);
    }
    for i in (0..outbuf.len()).step_by(64) {
        sum = sum.wrapping_add(upload[i] as u64);
    }
    black_box(sum);
    outbuf.len()
}

fn render_frame(frame_buf: &mut [u8], iters: u64) -> u64 {
    let mut sum = work(iters);
    for i in (0..frame_buf.len()).step_by(64) {
        let v = frame_buf[i].wrapping_add(sum as u8);
        frame_buf[i] = v;
        sum = sum.wrapping_add(v as u64);
    }
    sum
}

fn summarize_frames(mut frames: Vec<Duration>) -> (f64, usize) {
    let drops = frames.iter().filter(|&&wall| wall > FRAME_BUDGET).count();
    frames.sort_unstable();
    (frames[FRAMES * 99 / 100].as_secs_f64() * 1000.0, drops)
}

fn run_point(
    bank: &AssetBank,
    render_lp: usize,
    worker_lps: &[usize],
    workers_n: usize,
    pool_prio: ThreadPriority,
    work_iters: u64,
    render_priority_tally: &mut support::PriorityTally,
) -> WidthPoint {
    let stop = Arc::new(AtomicBool::new(false));
    let bytes = Arc::new(AtomicU64::new(0));
    let next_asset = Arc::new(AtomicU64::new(0));

    let mut workers = Vec::with_capacity(workers_n);
    for &lp in &worker_lps[..workers_n] {
        let stop = stop.clone();
        let bytes = bytes.clone();
        let next_asset = next_asset.clone();
        let bank = bank.clone();
        workers.push(thread::spawn(move || {
            let _ = pin_thread_to_core(lp);
            let _ = set_thread_priority(pool_prio);
            let mut outbuf = Vec::with_capacity(bank.asset_bytes);
            let mut upload = vec![0u8; bank.asset_bytes];
            while !stop.load(Ordering::Relaxed) {
                let idx = next_asset.fetch_add(1, Ordering::Relaxed) as usize % bank.assets.len();
                let n = decode_upload_and_touch(&bank.assets[idx], &mut outbuf, &mut upload);
                bytes.fetch_add(n as u64, Ordering::Relaxed);
            }
        }));
    }

    thread::sleep(Duration::from_millis(100));
    let start_bytes = bytes.load(Ordering::Relaxed);
    let start = Instant::now();
    let render = thread::spawn(move || {
        let _ = pin_thread_to_core(render_lp);
        let priority_result = set_thread_priority(ThreadPriority::AboveNormal);
        let mut frame_buf = vec![0u8; RENDER_BYTES];
        for (i, b) in frame_buf.iter_mut().enumerate() {
            *b = ((i * 33 + 17) % 251) as u8;
        }
        let mut frames = Vec::with_capacity(FRAMES);
        for _ in 0..FRAMES {
            let frame_start = Instant::now();
            black_box(render_frame(&mut frame_buf, work_iters));
            let wall = frame_start.elapsed();
            frames.push(wall);
            if let Some(slack) = FRAME_BUDGET.checked_sub(wall) {
                thread::sleep(slack);
            }
        }
        (priority_result, frames)
    })
    .join()
    .unwrap();
    let elapsed = start.elapsed();

    stop.store(true, Ordering::SeqCst);
    for worker in workers {
        let _ = worker.join();
    }

    render_priority_tally.record(render.0);
    let (p99_ms, drops) = summarize_frames(render.1);
    let measured_bytes = bytes.load(Ordering::Relaxed).saturating_sub(start_bytes);
    let gbps = (measured_bytes as f64 / 1_000_000_000.0) / elapsed.as_secs_f64();
    WidthPoint {
        workers: workers_n,
        gbps,
        p99_ms,
        drops,
    }
}

fn run_median_point(
    ctx: &RunContext<'_>,
    workers_n: usize,
    pool_prio: ThreadPriority,
    trials_n: usize,
    render_priority_tally: &mut support::PriorityTally,
) -> WidthPoint {
    let mut trials = Vec::with_capacity(trials_n);
    for _ in 0..trials_n {
        trials.push(run_point(
            ctx.bank,
            ctx.render_lp,
            ctx.worker_lps,
            workers_n,
            pool_prio,
            ctx.work_iters,
            render_priority_tally,
        ));
    }
    median_point(workers_n, &trials)
}

fn collect_candidate_lps(info: &gdt_cpus::CpuInfo) -> Vec<CandidateLp> {
    let mut candidates = info
        .lps
        .iter()
        .filter(|lp| lp.smt_index == 0 && lp.kind == CoreKind::Performance)
        .map(|lp| CandidateLp {
            os_id: lp.os_id as usize,
            l3_domain: lp.l3_domain,
        })
        .collect::<Vec<_>>();
    if candidates.len() >= 2 {
        return candidates;
    }
    candidates = info
        .lps
        .iter()
        .filter(|lp| lp.smt_index == 0)
        .map(|lp| CandidateLp {
            os_id: lp.os_id as usize,
            l3_domain: lp.l3_domain,
        })
        .collect();
    candidates
}

fn parse_asset_mib(arg: Option<String>) -> usize {
    arg.and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_ASSET_MIB)
        .max(1)
}

fn parse_trials(arg: Option<String>) -> usize {
    arg.and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_TRIALS)
        .clamp(1, MAX_TRIALS)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = gdt_cpus::CpuInfo::detect()?;
    let candidates = collect_candidate_lps(&info);
    if candidates.len() < 2 {
        eprintln!("background_budget: skipped; need at least 2 primary logical processors");
        return Ok(());
    }

    let render_lp = candidates[0].os_id;
    let worker_lps = round_robin_l3_lps(&candidates[1..]);

    let mut args = std::env::args().skip(1);
    let work_ms = args
        .next()
        .and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_WORK_MS);
    let asset_mib = parse_asset_mib(args.next());
    let trials_n = parse_trials(args.next());
    let asset_bytes = asset_mib * 1024 * 1024;
    let work_iters = calibrate(Duration::from_nanos((work_ms * 1_000_000.0) as u64));
    let asset_count = (candidates.len() * 2).clamp(MIN_ASSETS, MAX_ASSETS);

    println!(
        "{} - {} physical cores / {} threads",
        info.model_name,
        info.num_physical_cores(),
        info.num_logical_cores()
    );
    println!(
        "Render: lp {render_lp}, AboveNormal, 60 FPS, {work_ms:.1}ms ALU + {} MiB frame touch. Background work: {asset_count} x {asset_mib} MiB cold assets.",
        RENDER_BYTES / 1024 / 1024
    );
    println!(
        "Background workers: {} primary LPs after reserving render, L3 round-robin order, median of {trials_n} trial(s).\n",
        worker_lps.len()
    );

    let bank = build_asset_bank(asset_count, asset_bytes)?;
    let ctx = RunContext {
        bank: &bank,
        render_lp,
        worker_lps: &worker_lps,
        work_iters,
    };
    let widths = worker_width_schedule(worker_lps.len());
    let mut points = Vec::with_capacity(widths.len());
    let mut render_priority_tally = support::PriorityTally::new(ThreadPriority::AboveNormal);

    println!("\nwidth    throughput  frame p99    drops");
    for width in widths {
        let point = run_median_point(
            &ctx,
            width,
            ThreadPriority::Normal,
            trials_n,
            &mut render_priority_tally,
        );
        println!(
            "{:>5}  {:>7.2} GB/s  {:>7.2}ms  {:>3}/{FRAMES}",
            point.workers, point.gbps, point.p99_ms, point.drops
        );
        points.push(point);
    }

    let knee_i = choose_bandwidth_knee(&points);
    let knee = points[knee_i];
    let frame_safe = points[choose_frame_safe_pick(&points)];
    let (peak_i, _) = points
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.gbps.total_cmp(&b.gbps))
        .unwrap();
    let compare_i = if knee_i + 1 < points.len() {
        knee_i + 1
    } else {
        peak_i
    };
    let compare = points[compare_i];

    println!(
        "\nPriority check at comparison width ({} workers):",
        compare.workers
    );
    let mut priority_points = Vec::with_capacity(PRIORITY_SWEEP.len());
    for prio in PRIORITY_SWEEP {
        let point = if prio == ThreadPriority::Normal {
            compare
        } else {
            run_median_point(
                &ctx,
                compare.workers,
                prio,
                trials_n,
                &mut render_priority_tally,
            )
        };
        println!(
            "  pool @ {:<12}  {:>7.2} GB/s  p99 {:>7.2}ms  drops {:>3}/{FRAMES}",
            format!("{prio:?}"),
            point.gbps,
            point.p99_ms,
            point.drops
        );
        priority_points.push(point);
    }
    println!("  render priority: {}", render_priority_tally.render());

    let background = *priority_points.last().unwrap();
    println!(
        "\nFrame-safe budget: {} workers ({:.2} GB/s, p99 {:.2}ms, drops {}/{}); throughput knee {}; background priority at {} workers drops {}/{} -> {}/{}.",
        frame_safe.workers,
        frame_safe.gbps,
        frame_safe.p99_ms,
        frame_safe.drops,
        FRAMES,
        knee.workers,
        compare.workers,
        compare.drops,
        FRAMES,
        background.drops,
        FRAMES
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_width_schedule_doubles_to_cap() {
        assert_eq!(worker_width_schedule(12), vec![1, 2, 4, 8, 12]);
        assert_eq!(worker_width_schedule(15), vec![1, 2, 4, 8, 15]);
        assert_eq!(worker_width_schedule(1), vec![1]);
    }

    #[test]
    fn frame_safe_pick_chooses_fastest_zero_drop_point_with_headroom() {
        let points = [
            WidthPoint {
                workers: 1,
                gbps: 1.0,
                p99_ms: 4.0,
                drops: 0,
            },
            WidthPoint {
                workers: 4,
                gbps: 4.0,
                p99_ms: 12.0,
                drops: 0,
            },
            WidthPoint {
                workers: 8,
                gbps: 5.0,
                p99_ms: 15.4,
                drops: 0,
            },
            WidthPoint {
                workers: 15,
                gbps: 5.3,
                p99_ms: 16.2,
                drops: 1,
            },
        ];
        assert_eq!(choose_frame_safe_pick(&points), 1);
    }

    #[test]
    fn asset_mib_parser_has_no_upper_clamp() {
        assert_eq!(parse_asset_mib(Some("128".into())), 128);
        assert_eq!(parse_asset_mib(Some("0".into())), 1);
    }

    #[test]
    fn trial_parser_clamps_to_bounded_median_sample_range() {
        assert_eq!(parse_trials(Some("128".into())), 5);
        assert_eq!(parse_trials(Some("0".into())), 1);
    }

    #[test]
    fn l3_order_round_robins_domains() {
        let candidates = [
            CandidateLp {
                os_id: 0,
                l3_domain: 0,
            },
            CandidateLp {
                os_id: 1,
                l3_domain: 0,
            },
            CandidateLp {
                os_id: 8,
                l3_domain: 1,
            },
            CandidateLp {
                os_id: 9,
                l3_domain: 1,
            },
        ];
        assert_eq!(round_robin_l3_lps(&candidates), vec![0, 8, 1, 9]);
    }

    #[test]
    fn no_l3_domain_order_does_not_panic() {
        let candidates = [
            CandidateLp {
                os_id: 0,
                l3_domain: Lp::NO_L3,
            },
            CandidateLp {
                os_id: 1,
                l3_domain: Lp::NO_L3,
            },
        ];
        assert_eq!(round_robin_l3_lps(&candidates), vec![0, 1]);
    }
}
