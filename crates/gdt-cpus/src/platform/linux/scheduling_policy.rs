//! Linux-specific scheduling policy definitions.
//!
//! All seven [`ThreadPriority`](crate::ThreadPriority) levels map to `nice`
//! values under `SCHED_OTHER` ([`nice_for`]) - the default path never
//! requests a real-time class. Real-time scheduling (`SCHED_RR`,
//! [`RT_PRIORITY`]) exists only behind the explicit consent API
//! ([`crate::promote_thread_to_realtime`]).

use libc::c_int;

use crate::ThreadPriority;

// NOTE(linux): the ladder is symmetric (±5 / ±10 / ±max) and derived from the
// placement playbook. Each nice step is a ×1.25 CFS weight ratio, so the pairs read:
//   TimeCritical -20 ≈ ×9 over Highest   - the audio feeder must preempt even
//                                          your best threads on wake-up;
//   Highest      -10 ≈ ×9 over Normal    - a pinned render thread owns its
//                                          core without legally starving it;
//   AboveNormal   -5 ≈ ×3 over Normal    - responsive, not dominant;
//   BelowNormal    5 ≈ ÷3 of Normal      - streaming/decompression keeps
//                                          flowing under load (the old nice 10
//                                          was ÷9: Linux-only asset pop-in);
//   Lowest        10 ≈ ÷9 of Normal      - shader/PSO compiles, nav bakes,
//                                          batch work: real progress, scraps
//                                          under contention;
//   Background    19 ≈ ÷70 of Normal     - only-when-idle is the feature.

/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Background`.
pub const NICE_BACKGROUND: c_int = 19;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Lowest`.
pub const NICE_LOWEST: c_int = 10;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::BelowNormal`.
pub const NICE_BELOW_NORMAL: c_int = 5;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Normal`.
pub const NICE_NORMAL: c_int = 0;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::AboveNormal`.
pub const NICE_ABOVE_NORMAL: c_int = -5;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Highest`.
pub const NICE_HIGHEST: c_int = -10;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::TimeCritical`.
pub const NICE_TIME_CRITICAL: c_int = -20;

/// `SCHED_RR` priority used by the consent API
/// ([`crate::promote_thread_to_realtime`]) when direct promotion is possible.
///
/// NOTE(linux): the RT band has neighbors - threaded IRQs default to FIFO 50
/// (an "RT" game thread below that loses to every interrupt handler), desktop
/// audio daemons (PipeWire/JACK) run their data threads around RR 88, and the
/// kernel watchdog lives at 99. A game thread must sit ABOVE the IRQ threads
/// but BELOW the audio server that drains its buffers (producer never outranks
/// its consumer), and far from the watchdog - a spinning RR-99 thread can
/// wedge a core for 950 ms/s where RT is granted. Hence 85.
pub const RT_PRIORITY: c_int = 85;

/// The `nice` value for a given [`ThreadPriority`] under `SCHED_OTHER`:
/// Background -> 19, Lowest -> 10, BelowNormal -> 5, Normal -> 0, AboveNormal -> -5,
/// Highest -> -10, TimeCritical -> -20. Real-time scheduling is never the
/// default - see [`crate::promote_thread_to_realtime`].
pub const fn nice_for(priority: ThreadPriority) -> c_int {
    match priority {
        ThreadPriority::Background => NICE_BACKGROUND,
        ThreadPriority::Lowest => NICE_LOWEST,
        ThreadPriority::BelowNormal => NICE_BELOW_NORMAL,
        ThreadPriority::Normal => NICE_NORMAL,
        ThreadPriority::AboveNormal => NICE_ABOVE_NORMAL,
        ThreadPriority::Highest => NICE_HIGHEST,
        ThreadPriority::TimeCritical => NICE_TIME_CRITICAL,
    }
}

/// The [`ThreadPriority`] whose `nice` value is nearest to `nice` - the inverse
/// of [`nice_for`], rounded to the closest rung (ties favor the stronger /
/// lower-nice side). Used to report the level a thread ACTUALLY sits at when a
/// request could not be applied, so a denied request degrades to data ("you kept
/// the level you had") instead of an error.
pub fn level_for_nice(nice: c_int) -> ThreadPriority {
    const LADDER: [(c_int, ThreadPriority); 7] = [
        (NICE_TIME_CRITICAL, ThreadPriority::TimeCritical),
        (NICE_HIGHEST, ThreadPriority::Highest),
        (NICE_ABOVE_NORMAL, ThreadPriority::AboveNormal),
        (NICE_NORMAL, ThreadPriority::Normal),
        (NICE_BELOW_NORMAL, ThreadPriority::BelowNormal),
        (NICE_LOWEST, ThreadPriority::Lowest),
        (NICE_BACKGROUND, ThreadPriority::Background),
    ];

    LADDER
        .iter()
        .min_by_key(|(n, _)| (nice - n).abs())
        .map(|(_, level)| *level)
        .unwrap()
}
