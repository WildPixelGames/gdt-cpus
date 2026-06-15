//! Kernel range-list parsing ("0-3,7,10-11" -> CPU IDs).
//!
//! Platform-neutral on purpose: Linux production code parses `cpu/online`,
//! `cache/index*/shared_cpu_list` and `node*/cpulist` with it, and the
//! cross-platform fixture checker uses it for `expected.txt` LP lists.

use crate::{Error, Result};

/// Parses a kernel range-list string ("0-3,7,10-11"), calling `sink` once per
/// CPU id in order. Allocates nothing - the caller decides what to do with each
/// id (count it, set a mask bit, track a minimum), so the hot detection loops
/// that only need a membership test / count / min never materialize a throwaway
/// `Vec` per sysfs line.
///
/// # Errors
///
/// Returns `Error::Detection` on malformed ranges or non-numeric IDs.
pub(crate) fn parse_range_list_with<F: FnMut(usize)>(range_str: &str, mut sink: F) -> Result<()> {
    for part in range_str.trim().split(',') {
        let part = part.trim();

        if part.is_empty() {
            continue;
        }

        if part.contains('-') {
            let mut iter = part.splitn(2, '-');

            let start_str = iter
                .next()
                .ok_or_else(|| Error::Detection(format!("Invalid CPU range format: {}", part)))?;
            let end_str = iter
                .next()
                .ok_or_else(|| Error::Detection(format!("Invalid CPU range format: {}", part)))?;

            let start = start_str
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU range start: {}", start_str)))?;
            let end = end_str
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU range end: {}", end_str)))?;

            if start > end {
                return Err(Error::Detection(format!(
                    "Invalid CPU range order: {}-{}",
                    start, end
                )));
            }

            for id in start..=end {
                sink(id);
            }
        } else {
            let cpu_id = part
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU ID in range list: {}", part)))?;

            sink(cpu_id);
        }
    }

    Ok(())
}

/// Parses a kernel range-list string ("0-3,7,10-11") into CPU IDs.
///
/// Convenience over [`parse_range_list_with`] for the call sites that genuinely
/// need the materialized list (`cpu/online`, the fixture checker).
///
/// # Errors
///
/// Returns `Error::Detection` on malformed ranges or non-numeric IDs.
pub(crate) fn parse_range_list_str(range_str: &str) -> Result<Vec<usize>> {
    let mut cpus = Vec::new();

    parse_range_list_with(range_str, |id| cpus.push(id))?;

    Ok(cpus)
}
