use super::CoreKind;

/// One record per ONLINE logical processor - the flat topology's atom.
///
/// The whole machine is described by `Vec<Lp>` plus derived counts and the
/// L3-domain table; there is no socket -> core nesting (a per-socket hierarchy
/// cannot represent chiplet CPUs, where one socket carries several L3 domains).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lp {
    /// OS logical-processor id. Affinity masks address THESE ids - they may be
    /// sparse (offline CPUs) and are never remapped by the library.
    pub os_id: u16,
    /// Dense library-assigned physical core index, `0..core_count`.
    pub core: u16,
    /// Dense socket index.
    pub socket: u8,
    /// Index into [`crate::CpuInfo::l3_domains`], or [`Lp::NO_L3`] when the LP
    /// reports no L3 (e.g. Apple Silicon) or cache detection found none.
    pub l3_domain: u8,
    /// OS NUMA node id (0 on single-node systems and macOS).
    pub numa_node: u8,
    /// Performance/efficiency classification of this LP's physical core.
    pub kind: CoreKind,
    /// 0 = first SMT sibling on its physical core.
    pub smt_index: u8,
    /// Relative performance hint - ORDINAL and MACHINE-LOCAL: a higher value
    /// means a faster core *on this machine*, equal values are
    /// indistinguishable, and the scale differs per OS (Linux: kernel
    /// `cpu_capacity`, 0-1024; Windows: GLPI `EfficiencyClass`; macOS:
    /// perflevel order). 0 = no finer signal than [`Lp::kind`].
    ///
    /// The intended use: picking the BEST cores within a kind - e.g. a chip
    /// whose Performance tier spans several frequency bins (Intel ITMT
    /// favored cores, ARM DynamIQ prime-vs-mid):
    /// `lps.iter().filter(|l| l.kind == CoreKind::Performance)
    ///     .max_by_key(|l| l.perf_hint)`.
    pub perf_hint: u16,
    /// Raw ARM MIDR part number of this core's microarchitecture (e.g. `0x0d0b`
    /// = Cortex-A76, `0x0d81` = Cortex-A720, `0x0d80` = Cortex-A520), read
    /// per-core from `/proc/cpuinfo`. `0` when no such field exists (x86, where
    /// identity lives in [`crate::CpuInfo::model_name`]) or it was not reported.
    ///
    /// Combined with the chip vendor (the MIDR *implementer*, surfaced as
    /// [`crate::CpuInfo::vendor`]) the part uniquely names the microarchitecture.
    /// It is exposed raw so a caller can tell cores of different microarchitectures
    /// apart (e.g. pin work off the little cores) without the library interpreting them.
    /// It is NOT a kind signal: classification uses [`Lp::kind`] / [`Lp::perf_hint`],
    /// which are vendor-neutral.
    pub cpu_part: u16,
}

impl Lp {
    /// Sentinel for [`Lp::l3_domain`]: this LP belongs to no detected L3 domain.
    pub const NO_L3: u8 = 0xFF;
}
