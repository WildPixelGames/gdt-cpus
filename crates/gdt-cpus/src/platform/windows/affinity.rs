//! Windows thread affinity (hard + soft) and priority for the current thread.
//!
//! Hard affinity uses `SetThreadGroupAffinity`: a thread's hard affinity is
//! single-group by OS design, so masks spanning multiple 64-LP processor
//! groups are rejected (`Error::InvalidParameter`) - CPU Sets are the
//! cross-group tool. Soft affinity (`set_thread_soft_affinity`) uses the CPU
//! Sets API (`SetThreadSelectedCpuSets`), the scheduling mode Intel's game
//! guidance recommends: the scheduler PREFERS the given LPs but may still
//! migrate under contention, cooperating with Thread Director / parking.

use windows::Win32::Foundation::{HANDLE, NTSTATUS};
use windows::Win32::System::SystemInformation::{GROUP_AFFINITY, GetSystemCpuSetInformation};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, SetThreadGroupAffinity, SetThreadPriority,
    SetThreadSelectedCpuSets, THREAD_PRIORITY,
};

use super::scheduling_policy::SchedulingPolicy;
use crate::{
    AffinityMask, AppliedPriority, Error, Grant, Mechanism, MechanismPolicy, Result, ThreadPriority,
};

// THREADINFOCLASS::ThreadGroupInformation - reads the thread's hard
// GROUP_AFFINITY. The Win32 surface (`Win32_System_Threading`) exposes the
// SetThreadGroupAffinity *setter* but no symmetric getter; ntdll's
// NtQueryInformationThread is the documented read path, declared directly here
// rather than pulling in the Wdk crate feature for one call.
const THREAD_GROUP_INFORMATION: i32 = 22;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationThread(
        ThreadHandle: HANDLE,
        ThreadInformationClass: i32,
        ThreadInformation: *mut core::ffi::c_void,
        ThreadInformationLength: u32,
        ReturnLength: *mut u32,
    ) -> NTSTATUS;
}

/// Sets the current thread's HARD affinity to `mask` (OS LP ids).
///
/// The mask must stay within ONE 64-LP processor group (`group = os_id / 64`);
/// that is an OS rule, not a library limitation. Multi-group placement is what
/// [`set_thread_soft_affinity`] is for.
pub(crate) fn set_thread_affinity(mask: &AffinityMask) -> Result<()> {
    if mask.is_empty() {
        return Err(Error::Affinity(
            "Cannot set thread affinity with an empty mask".to_string(),
        ));
    }

    // Split by processor group; exactly one group may carry bits.
    let mut group: Option<u16> = None;
    let mut bits: usize = 0;

    for lp in mask.iter() {
        let g = (lp / 64) as u16;

        match group {
            None => group = Some(g),
            Some(existing) if existing != g => {
                return Err(Error::InvalidParameter(format!(
                    "Hard affinity is single-group on Windows (mask spans groups {} and {}); \
                     use soft affinity (CPU Sets) for cross-group placement",
                    existing, g
                )));
            }
            _ => {}
        }

        bits |= 1usize << (lp % 64);
    }

    let ga = GROUP_AFFINITY {
        Mask: bits,
        Group: group.unwrap_or(0),
        Reserved: [0; 3],
    };

    let ok = unsafe { SetThreadGroupAffinity(GetCurrentThread(), &ga, None) };

    if !ok.as_bool() {
        let err = std::io::Error::last_os_error();
        return Err(Error::Affinity(format!(
            "SetThreadGroupAffinity failed: {}",
            err
        )));
    }

    Ok(())
}

/// Reads the calling thread's hard-affinity mask into an [`AffinityMask`] via
/// `NtQueryInformationThread(ThreadGroupInformation)` (one `GROUP_AFFINITY`).
///
/// A thread's hard affinity is single-group by OS design, so the result carries
/// bits from exactly one processor group (`group * 64 + bit`).
pub(crate) fn current_affinity() -> Result<AffinityMask> {
    let mut ga = GROUP_AFFINITY::default();
    let mut ret_len: u32 = 0;

    // SAFETY: a valid thread handle, a properly sized GROUP_AFFINITY out-buffer,
    // and a ReturnLength pointer; ntdll writes at most size_of::<GROUP_AFFINITY>.
    let status = unsafe {
        NtQueryInformationThread(
            GetCurrentThread(),
            THREAD_GROUP_INFORMATION,
            (&raw mut ga).cast(),
            std::mem::size_of::<GROUP_AFFINITY>() as u32,
            &mut ret_len,
        )
    };

    if status.is_err() {
        return Err(Error::Affinity(format!(
            "NtQueryInformationThread(ThreadGroupInformation) failed: {status:?}"
        )));
    }

    let mut mask = AffinityMask::empty();

    for bit in 0..64usize {
        if (ga.Mask >> bit) & 1 != 0 {
            mask.add(ga.Group as usize * 64 + bit);
        }
    }

    Ok(mask)
}

/// Sets the current thread's SOFT affinity (CPU Sets) to `mask` (OS LP ids).
///
/// Cross-group capable. Never called with an empty selection - passing zero
/// CpuSet ids to the OS would CLEAR the assignment, which v1 does not expose.
pub(crate) fn set_thread_soft_affinity(mask: &AffinityMask) -> Result<()> {
    if mask.is_empty() {
        return Err(Error::Affinity(
            "Cannot set soft affinity with an empty mask".to_string(),
        ));
    }

    // Walk SYSTEM_CPU_SET_INFORMATION records; map Group*64+LogicalProcessorIndex
    // to OS LP ids and collect the CpuSet Ids covered by the mask.
    let mut needed: u32 = 0;

    unsafe {
        let _ = GetSystemCpuSetInformation(None, 0, &mut needed, Some(GetCurrentProcess()), None);
    }

    if needed == 0 {
        return Err(Error::Unsupported(
            "CPU Sets are not available on this system".to_string(),
        ));
    }

    let mut buffer: Vec<u8> = vec![0; needed as usize];

    let ok = unsafe {
        GetSystemCpuSetInformation(
            Some(buffer.as_mut_ptr() as *mut _),
            needed,
            &mut needed,
            Some(GetCurrentProcess()),
            None,
        )
    };

    if !ok.as_bool() {
        let err = std::io::Error::last_os_error();
        return Err(Error::SystemCall(format!(
            "GetSystemCpuSetInformation failed: {}",
            err
        )));
    }

    let mut ids: Vec<u32> = Vec::new();
    let mut offset: usize = 0;

    while offset + 8 <= needed as usize {
        // Size (u32) + Type (u32) prefix; CpuSet payload follows.
        let size = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;

        // Reject malformed records: `size` in 1..8 would make the payload slice
        // start (offset+8) past its end (offset+size) and panic; `size == 0`
        // would loop forever. Both are caught by requiring a full 8-byte header.
        if size < 8 || offset + size > needed as usize {
            break;
        }

        // `Type` is the second u32. This is an extensible tagged stream;
        // CpuSetInformation == 0. Skip any other record type (do not parse its
        // bytes as a CpuSet) but keep walking.
        let rec_type = u32::from_le_bytes(buffer[offset + 4..offset + 8].try_into().unwrap());

        if rec_type != 0 {
            offset += size;
            continue;
        }

        let payload = &buffer[offset + 8..offset + size];

        // SYSTEM_CPU_SET_INFORMATION.CpuSet: Id(u32) Group(u16) LpIndex(u8) ...
        if payload.len() >= 8 {
            let id = u32::from_le_bytes(payload[0..4].try_into().unwrap());
            let group = u16::from_le_bytes(payload[4..6].try_into().unwrap());
            let lp_index = payload[6];
            let os_id = group as usize * 64 + lp_index as usize;

            if mask.contains(os_id) {
                ids.push(id);
            }
        }

        offset += size;
    }

    if ids.is_empty() {
        return Err(Error::InvalidCoreId(
            mask.iter().next().unwrap_or(usize::MAX),
        ));
    }

    let ok = unsafe { SetThreadSelectedCpuSets(GetCurrentThread(), &ids) };

    if !ok.as_bool() {
        let err = std::io::Error::last_os_error();

        return Err(Error::SystemCall(format!(
            "SetThreadSelectedCpuSets failed: {}",
            err
        )));
    }

    Ok(())
}

/// Sets the current thread's priority (`THREAD_PRIORITY_IDLE..TIME_CRITICAL`).
pub(crate) fn set_thread_priority(priority: ThreadPriority) -> Result<AppliedPriority> {
    set_thread_priority_with_grant(priority, Grant::Direct)
}

/// Applies Windows time-critical priority through the explicit realtime API.
pub(crate) fn promote_thread_to_realtime() -> Result<AppliedPriority> {
    set_thread_priority_with_grant(ThreadPriority::TimeCritical, Grant::Realtime)
}

fn set_thread_priority_with_grant(
    priority: ThreadPriority,
    grant: Grant,
) -> Result<AppliedPriority> {
    // Map the portable level to the Windows value (table in scheduling_policy.rs).
    let sched_policy = SchedulingPolicy::default_for(priority);

    let result = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY(sched_policy.0)) };

    match result {
        Ok(_) => Ok(AppliedPriority::new(
            priority,
            priority,
            grant,
            // Deterministic mapping, no privilege degradation - Windows always
            // delivers exactly the requested level (failures are Err below).
            Mechanism {
                policy: MechanismPolicy::WinPriority,
                value: sched_policy.0 as i8,
            },
        )),
        Err(e) => Err(Error::SystemCall(format!(
            "SetThreadPriority failed with error: {:?}",
            e
        ))),
    }
}
