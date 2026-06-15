//! Demonstrates the thread-priority introspection API - no benchmarking, just
//! "what can this process currently get, and what did each request actually do":
//!
//! * [`priority_capabilities`] - predict, BEFORE touching any thread, how many
//!   ladder levels are effectively distinct here (the ladder collapses without
//!   privilege / rtkit, and a render thread that can't outrank a worker is a
//!   thing you want to know at startup, not from frame times).
//! * [`set_thread_priority`] -> [`AppliedPriority`] - what each request actually
//!   produced: direct nice, an rtkit grant, real-time, or a silent fallback to
//!   Normal (`degraded()`).
//! * [`promote_thread_to_realtime`] / [`demote_thread_from_realtime`] - the
//!   explicit real-time opt-in.
//!
//! On Linux, compare unprivileged vs privileged on the SAME box:
//! ```text
//! cargo build --example thread_priorities
//! ./target/debug/examples/thread_priorities                       # rtkit / fallbacks
//! sudo setcap cap_sys_nice+ep target/debug/examples/thread_priorities
//! ./target/debug/examples/thread_priorities                       # direct grants
//! ```

use gdt_cpus::{
    AppliedPriority, ThreadPriority, demote_thread_from_realtime, priority_capabilities,
    promote_thread_to_realtime, set_thread_priority,
};
use std::time::Duration;

const LEVELS: [ThreadPriority; 7] = [
    ThreadPriority::Background,
    ThreadPriority::Lowest,
    ThreadPriority::BelowNormal,
    ThreadPriority::Normal,
    ThreadPriority::AboveNormal,
    ThreadPriority::Highest,
    ThreadPriority::TimeCritical,
];

fn main() {
    // 1. Pre-flight PREDICTION - touches no thread, costs microseconds.
    let caps = priority_capabilities();
    println!("priority_capabilities() - what this process can currently get:");
    println!("  effective ranks : {:?}", caps.effective_rank);
    println!("  distinct levels : {}/7", caps.distinct_levels());
    println!("  row detail      : concrete scheduler API and value applied below");
    if caps.distinct(ThreadPriority::Highest, ThreadPriority::Normal) {
        println!("  verdict         : the full ladder is effectively distinct here.");
    } else {
        println!("  verdict         : Highest == Normal - a render thread will NOT");
        println!("                    outrank your workers (no privilege / no rtkit).");
    }

    // 2. Observed OUTCOME - set each level on its own fresh thread (so nice /
    //    policy changes don't accumulate) and report exactly what stuck.
    println!("\nset_thread_priority() - what each request actually does:");
    for level in LEVELS {
        let applied: gdt_cpus::Result<AppliedPriority> =
            std::thread::spawn(move || set_thread_priority(level))
                .join()
                .unwrap();
        match applied {
            Ok(a) => println!("  {a}"),
            Err(e) => println!("  {level}: error: {e}"),
        }
    }

    // 3. The explicit real-time opt-in (and its self-demotion partner).
    println!("\npromote_thread_to_realtime() - the consent API:");
    std::thread::spawn(|| {
        match promote_thread_to_realtime(Duration::from_millis(1)) {
            Ok(a) => {
                println!("  promoted : {a}");
                match demote_thread_from_realtime() {
                    Ok(()) => println!("  demoted  : back to normal scheduling"),
                    Err(e) => println!("  demote failed: {e}"),
                }
            }
            // Denied is a legitimate outcome on a box with neither privilege
            // nor a reachable broker - the consent API reports it instead of
            // silently degrading (unlike set_thread_priority).
            Err(e) => println!("  denied   : {e}"),
        }
    })
    .join()
    .unwrap();
}
