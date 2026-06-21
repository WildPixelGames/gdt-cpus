//! Windows CPU topology detection - two-phase GLPI walk into the flat model.
//!
//! Phase 1 COLLECTS raw relations from one `GetLogicalProcessorInformationEx`
//! buffer (cores with their RELATIVE `EfficiencyClass`, packages, caches, NUMA
//! nodes). Phase 2 CLASSIFIES kinds (EfficiencyClass is relative - all classes
//! must be seen before any core can be labeled) and only then BUCKETS per-kind
//! caches. Do not merge the phases back into one pass: single-pass bucketing
//! uses kinds that don't exist yet.
//!
//! LP ids follow the `group * 64 + bit` convention. L3 domains are
//! content-keyed by their lowest member LP - never attributed per socket:
//! chiplet CPUs report several L3 relations per package, and every one of
//! them is a domain. `RelationNumaNode` is consumed, not skipped.

use std::ptr;

use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::SystemInformation::{
    CacheData, CacheInstruction, CacheUnified, GetLogicalProcessorInformationEx, RelationAll,
    RelationCache, RelationNumaNode, RelationProcessorCore, RelationProcessorPackage,
    SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
};

#[cfg(target_arch = "aarch64")]
use windows::Win32::System::Threading::{
    IsProcessorFeaturePresent, PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE,
    PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE, PF_ARM_V8_INSTRUCTIONS_AVAILABLE,
    PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE, PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE,
    PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE, PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE,
    PROCESSOR_FEATURE_ID,
};

use crate::{
    AffinityMask, CacheInfo, CoreKind, CpuFeatures, CpuInfo, Error, L2Domain, L3Domain, Lp, Result,
    Vendor,
};

#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(46);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(47);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(49);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID =
    PROCESSOR_FEATURE_ID(50);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID =
    PROCESSOR_FEATURE_ID(51);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_BF16_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(52);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(55);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(56);
#[cfg(target_arch = "aarch64")]
const PF_ARM_SVE_I8MM_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(57);
#[cfg(target_arch = "aarch64")]
const PF_ARM_LSE2_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(62);
#[cfg(target_arch = "aarch64")]
const PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(66);
#[cfg(target_arch = "aarch64")]
const PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(67);
#[cfg(target_arch = "aarch64")]
const PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE_LOCAL: PROCESSOR_FEATURE_ID = PROCESSOR_FEATURE_ID(68);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheLevel {
    L1,
    L2,
    L3,
    L4,
    Unknown,
}

impl From<u32> for CacheLevel {
    fn from(level: u32) -> Self {
        match level {
            1 => CacheLevel::L1,
            2 => CacheLevel::L2,
            3 => CacheLevel::L3,
            4 => CacheLevel::L4,
            _ => CacheLevel::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheType {
    Unified,
    Instruction,
    Data,
    Unknown,
}

struct RawCore {
    efficiency_class: u8,
    lp_ids: Vec<u16>, // sorted ascending
}

struct RawCache {
    level: CacheLevel,
    cache_type: CacheType,
    size_bytes: u64,
    line_bytes: u16,
    lp_ids: Vec<u16>,
}

#[derive(Default)]
struct RawTopology {
    cores: Vec<RawCore>,
    packages: Vec<Vec<u16>>,
    caches: Vec<RawCache>,
    numa_nodes: Vec<(u32, Vec<u16>)>,
}

/// Expands a GROUP_AFFINITY into `group * 64 + bit` LP ids.
fn expand_group_mask(group: u16, mask: usize, out: &mut Vec<u16>) {
    for bit in 0..(std::mem::size_of::<usize>() * 8) {
        if (mask >> bit) & 1 != 0 {
            out.push((group as usize * 64 + bit) as u16);
        }
    }
}

/// Detects CPU information on Windows.
pub fn detect_cpu_info() -> Result<CpuInfo> {
    // --- Identity ---
    let mut vendor = Vendor::Unknown;
    let mut model_name = "Unknown".to_string();
    let mut features = CpuFeatures::default();

    #[cfg(target_arch = "x86_64")]
    crate::platform::common_x86_64::detect_via_cpuid(&mut vendor, &mut model_name, &mut features);

    #[cfg(target_arch = "aarch64")]
    {
        if vendor == Vendor::Unknown {
            vendor = Vendor::Arm;
        }
        detect_features_via_processor_feature(&mut features);
    }

    if vendor == Vendor::Unknown || model_name == "Unknown" {
        super::registry::detect_via_registry(&mut vendor, &mut model_name)?;
    }

    // --- GLPI buffer, two-call pattern ---
    let mut buffer_size: u32 = 0;

    let size_query = unsafe {
        GetLogicalProcessorInformationEx(RelationAll, Some(ptr::null_mut()), &mut buffer_size)
    };

    if size_query.is_err() && buffer_size == 0 {
        let err = unsafe { GetLastError() };

        return Err(Error::SystemCall(format!(
            "GetLogicalProcessorInformationEx (size query) failed: {:?}",
            err
        )));
    }

    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

    unsafe {
        GetLogicalProcessorInformationEx(
            RelationAll,
            Some(buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX),
            &mut buffer_size,
        )
    }
    .map_err(|e| Error::SystemCall(format!("GetLogicalProcessorInformationEx failed: {:?}", e)))?;

    // --- Phase 1: collect raw relations (no classification yet) ---
    let mut raw = RawTopology::default();

    let mut current = buffer.as_ptr();

    let end = unsafe { buffer.as_ptr().add(buffer_size as usize) };

    #[allow(non_upper_case_globals)]
    while current < end {
        let info = unsafe { &*(current as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) };

        if info.Size == 0 {
            // A zero stride would loop forever; a malformed/unknown record with
            // Size == 0 stops the walk instead of hanging detection.
            break;
        }

        match info.Relationship {
            RelationProcessorCore | RelationProcessorPackage => {
                let rel = unsafe { &info.Anonymous.Processor };

                // ANYSIZE fan-out: GroupCount entries live past the declared [1].
                let count = (rel.GroupCount as usize).max(1);
                let masks = unsafe { std::slice::from_raw_parts(rel.GroupMask.as_ptr(), count) };

                let mut lp_ids = Vec::new();

                for ga in masks {
                    expand_group_mask(ga.Group, ga.Mask, &mut lp_ids);
                }

                lp_ids.sort_unstable();

                if info.Relationship == RelationProcessorCore {
                    raw.cores.push(RawCore {
                        efficiency_class: rel.EfficiencyClass,
                        lp_ids,
                    });
                } else {
                    raw.packages.push(lp_ids);
                }
            }
            RelationCache => {
                let rel = unsafe { &info.Anonymous.Cache };
                let cache_type = match rel.Type {
                    CacheUnified => CacheType::Unified,
                    CacheInstruction => CacheType::Instruction,
                    CacheData => CacheType::Data,
                    _ => CacheType::Unknown,
                };

                // GroupCount == 0 => one legacy GroupMask (pre-Win10-20H2).
                let count = (rel.GroupCount as usize).max(1);

                let mut lp_ids = Vec::new();

                unsafe {
                    let masks =
                        std::slice::from_raw_parts(rel.Anonymous.GroupMasks.as_ptr(), count);
                    for ga in masks {
                        expand_group_mask(ga.Group, ga.Mask, &mut lp_ids);
                    }
                }

                lp_ids.sort_unstable();

                raw.caches.push(RawCache {
                    level: CacheLevel::from(rel.Level as u32),
                    cache_type,
                    size_bytes: rel.CacheSize as u64,
                    line_bytes: rel.LineSize,
                    lp_ids,
                });
            }
            RelationNumaNode => {
                let rel = unsafe { &info.Anonymous.NumaNode };
                let count = (rel.GroupCount as usize).max(1);

                let mut lp_ids = Vec::new();

                unsafe {
                    let masks =
                        std::slice::from_raw_parts(rel.Anonymous.GroupMasks.as_ptr(), count);
                    for ga in masks {
                        expand_group_mask(ga.Group, ga.Mask, &mut lp_ids);
                    }
                }

                raw.numa_nodes.push((rel.NodeNumber, lp_ids));
            }
            _ => {}
        }
        current = unsafe { current.add(info.Size as usize) };
    }

    build_cpu_info(raw, vendor, model_name, features)
}

fn build_cpu_info(
    raw: RawTopology,
    vendor: Vendor,
    model_name: String,
    features: CpuFeatures,
) -> Result<CpuInfo> {
    if raw.cores.is_empty() {
        return Err(Error::Detection(
            "GetLogicalProcessorInformationEx reported no processor cores".to_string(),
        ));
    }

    // --- Phase 2a: classify kinds. EfficiencyClass is RELATIVE: higher =
    // more performant; homogeneous machines report all-zero. k distinct
    // classes => k==1: all Performance; k==2: max=P, min=E; k>=3: max=P,
    // min=LP-E, middle=E.
    let mut classes: Vec<u8> = raw.cores.iter().map(|c| c.efficiency_class).collect();

    classes.sort_unstable();
    classes.dedup();

    let kind_of_class = |class: u8| -> CoreKind {
        match classes.len() {
            0 | 1 => CoreKind::Performance,
            2 => {
                if class == *classes.last().unwrap() {
                    CoreKind::Performance
                } else {
                    CoreKind::Efficiency
                }
            }
            _ => {
                if class == *classes.last().unwrap() {
                    CoreKind::Performance
                } else if class == classes[0] {
                    CoreKind::LpEfficiency
                } else {
                    CoreKind::Efficiency
                }
            }
        }
    };

    // --- Phase 2b: LP records (dense cores, sockets by package membership) ---
    let mut lps: Vec<Lp> = Vec::new();

    for (core_idx, core) in raw.cores.iter().enumerate() {
        let kind = kind_of_class(core.efficiency_class);

        for (sibling, &os_id) in core.lp_ids.iter().enumerate() {
            let socket = raw
                .packages
                .iter()
                .position(|pkg| pkg.binary_search(&os_id).is_ok())
                .unwrap_or(0) as u8;

            lps.push(Lp {
                os_id,
                core: core_idx as u16,
                socket,
                l3_domain: Lp::NO_L3,
                l2_domain: Lp::NO_L2,
                numa_node: 0,
                kind,
                smt_index: sibling as u8,
                // GLPI EfficiencyClass is RELATIVE, higher = more performant -
                // already exactly the ordinal machine-local hint contract.
                perf_hint: core.efficiency_class as u16,
                // NOTE(windows): GLPI exposes no per-core MIDR part; Windows-on-ARM
                // identifies cores by EfficiencyClass, not microarch id. Leave 0.
                cpu_part: 0,
            });
        }
    }

    lps.sort_unstable_by_key(|lp| lp.os_id);

    let core_count = raw.cores.len() as u16;
    let socket_count = raw.packages.len().max(1) as u8;

    // --- Phase 2c: L3 domains, content-keyed by lowest member LP ---
    let mut l3_domains: Vec<L3Domain> = Vec::new();
    let mut domain_first_lp: Vec<u16> = Vec::new();

    for cache in raw
        .caches
        .iter()
        .filter(|c| c.level == CacheLevel::L3 && c.cache_type == CacheType::Unified)
    {
        let Some(&first) = cache.lp_ids.first() else {
            continue;
        };

        let domain = match domain_first_lp.iter().position(|&k| k == first) {
            Some(d) => d,
            None => {
                // A new L3 domain. Cap BEFORE pushing: NO_L3 (255) is the
                // "no L3 domain" sentinel, so the last usable index is 254.
                // Rejecting here keeps a real domain from being labelled L3-less
                // (index == NO_L3) and never appends a phantom domain.
                if l3_domains.len() >= Lp::NO_L3 as usize {
                    continue;
                }

                let mut mask = AffinityMask::empty();

                for &id in &cache.lp_ids {
                    mask.add(id as usize);
                }

                domain_first_lp.push(first);

                l3_domains.push(L3Domain {
                    size_bytes: cache.size_bytes,
                    mask,
                    core_count: 0,
                });

                l3_domains.len() - 1
            }
        };

        for lp in lps.iter_mut() {
            if cache.lp_ids.binary_search(&lp.os_id).is_ok() {
                lp.l3_domain = domain as u8;
            }
        }
    }

    for lp in &lps {
        if lp.smt_index == 0 && lp.l3_domain != Lp::NO_L3 {
            l3_domains[lp.l3_domain as usize].core_count += 1;
        }
    }

    // --- Phase 2c-bis: L2 domains, content-keyed by lowest member LP ---
    let mut l2_domains: Vec<L2Domain> = Vec::new();
    let mut l2_first_lp: Vec<u16> = Vec::new();

    for cache in raw
        .caches
        .iter()
        .filter(|c| c.level == CacheLevel::L2 && c.cache_type == CacheType::Unified)
    {
        let Some(&first) = cache.lp_ids.first() else {
            continue;
        };

        let domain = match l2_first_lp.iter().position(|&k| k == first) {
            Some(d) => d,
            None => {
                // Cap BEFORE pushing: NO_L2 (0xFFFF) is the sentinel.
                if l2_domains.len() >= Lp::NO_L2 as usize {
                    continue;
                }

                let mut mask = AffinityMask::empty();
                for &id in &cache.lp_ids {
                    mask.add(id as usize);
                }

                // Every member shares one L3; take it from the lowest member,
                // already stamped by phase 2c.
                let l3_domain = lps
                    .iter()
                    .find(|lp| lp.os_id == first)
                    .map_or(Lp::NO_L3, |lp| lp.l3_domain);

                l2_first_lp.push(first);

                l2_domains.push(L2Domain {
                    size_bytes: cache.size_bytes,
                    mask,
                    core_count: 0,
                    l3_domain,
                });

                l2_domains.len() - 1
            }
        };

        for lp in lps.iter_mut() {
            if cache.lp_ids.binary_search(&lp.os_id).is_ok() {
                lp.l2_domain = domain as u16;
            }
        }
    }

    for lp in &lps {
        if lp.smt_index == 0 && lp.l2_domain != Lp::NO_L2 {
            l2_domains[lp.l2_domain as usize].core_count += 1;
        }
    }

    // --- Phase 2d: per-kind L1/L2 buckets (kinds are final now) ---
    let mut l1d = [CacheInfo::default(); CoreKind::COUNT];
    let mut l1i = [CacheInfo::default(); CoreKind::COUNT];
    let mut l2 = [CacheInfo::default(); CoreKind::COUNT];

    for lp in lps.iter().filter(|lp| lp.smt_index == 0) {
        let k = lp.kind.index();

        if l1d[k].size_bytes != 0 && l2[k].size_bytes != 0 {
            continue;
        }

        for cache in raw
            .caches
            .iter()
            .filter(|c| c.level != CacheLevel::L3 && c.lp_ids.binary_search(&lp.os_id).is_ok())
        {
            let ci = CacheInfo {
                size_bytes: cache.size_bytes,
                line_bytes: cache.line_bytes,
                shared_by: cache.lp_ids.len() as u16,
            };

            match (cache.level, cache.cache_type) {
                (CacheLevel::L2, _) => l2[k] = ci,
                (CacheLevel::L1, CacheType::Data) => l1d[k] = ci,
                (CacheLevel::L1, CacheType::Instruction) => l1i[k] = ci,
                (CacheLevel::L1, CacheType::Unified) => {
                    l1d[k] = ci;
                    l1i[k] = ci;
                }
                _ => {}
            }
        }
    }

    // --- Phase 2e: NUMA stamping ---
    let mut numa_node_count: u8 = 1;

    for (node, lp_ids) in &raw.numa_nodes {
        let node_u8 = (*node).min(u8::MAX as u32) as u8;

        for lp in lps.iter_mut() {
            if lp_ids.contains(&lp.os_id) {
                lp.numa_node = node_u8;
            }
        }
    }

    // Count DISTINCT present NodeNumbers, not the relation count: a sparse or
    // duplicated NodeNumber set (nodes {0, 2}, or one node split across two
    // relations) must report the true number of memory domains. Relation `len()`
    // over-counts duplicates; max-id+1 over-counts gaps.
    if !raw.numa_nodes.is_empty() {
        let mut distinct: Vec<u32> = raw.numa_nodes.iter().map(|(node, _)| *node).collect();

        distinct.sort_unstable();
        distinct.dedup();

        numa_node_count = distinct.len().min(u8::MAX as usize) as u8;
    }

    // --- Counts ---
    let mut kind_core_counts = [0u16; CoreKind::COUNT];

    for lp in &lps {
        if lp.smt_index == 0 {
            kind_core_counts[lp.kind.index()] += 1;
        }
    }

    let mut info = CpuInfo {
        lps,
        core_count,
        socket_count,
        numa_node_count,
        kind_core_counts,
        l3_domains,
        l2_domains,
        l1d,
        l1i,
        l2,
        vendor,
        model_name,
        features,
    };
    info.normalize_domain_order();
    Ok(info)
}

#[cfg(target_arch = "aarch64")]
fn processor_feature(feature: windows::Win32::System::Threading::PROCESSOR_FEATURE_ID) -> bool {
    unsafe { IsProcessorFeaturePresent(feature).as_bool() }
}

#[cfg(target_arch = "aarch64")]
fn detect_features_via_processor_feature(features: &mut CpuFeatures) {
    if processor_feature(PF_ARM_V8_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::NEON);
    }
    if processor_feature(PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::AES | CpuFeatures::SHA | CpuFeatures::PMULL);
    }
    if processor_feature(PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::CRC32);
    }
    if processor_feature(PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::LSE);
    }
    if processor_feature(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::DOTPROD);
    }
    if processor_feature(PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::FP16);
    }
    if processor_feature(PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::I8MM);
    }
    if processor_feature(PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::JSCVT);
    }
    if processor_feature(PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE) {
        features.insert(CpuFeatures::LRCPC);
    }
    if processor_feature(PF_ARM_SVE_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVE);
    }
    if processor_feature(PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVE2);
    }
    if processor_feature(PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVEAES);
    }
    if processor_feature(PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVEPMULL);
    }
    if processor_feature(PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVEBITPERM);
    }
    if processor_feature(PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVESHA3);
    }
    if processor_feature(PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVESM4);
    }
    if processor_feature(PF_ARM_SVE_I8MM_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVEI8MM);
    }
    if processor_feature(PF_ARM_SVE_BF16_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::SVEBF16);
    }
    if processor_feature(PF_ARM_LSE2_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::LSE2);
    }
    if processor_feature(PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE_LOCAL) {
        features.insert(CpuFeatures::BF16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core(efficiency_class: u8, lp_ids: &[u16]) -> RawCore {
        RawCore {
            efficiency_class,
            lp_ids: lp_ids.to_vec(),
        }
    }

    fn cache(
        level: CacheLevel,
        cache_type: CacheType,
        size_bytes: u64,
        lp_ids: &[u16],
    ) -> RawCache {
        RawCache {
            level,
            cache_type,
            size_bytes,
            line_bytes: 64,
            lp_ids: lp_ids.to_vec(),
        }
    }

    fn info(raw: RawTopology) -> CpuInfo {
        build_cpu_info(
            raw,
            Vendor::Unknown,
            "test cpu".to_string(),
            CpuFeatures::default(),
        )
        .unwrap()
    }

    #[test]
    fn two_efficiency_classes_are_efficiency_and_performance() {
        let raw = RawTopology {
            cores: vec![core(0, &[0]), core(1, &[1])],
            packages: vec![vec![0, 1]],
            ..RawTopology::default()
        };
        let info = info(raw);

        assert_eq!(info.kind_core_counts[CoreKind::Efficiency.index()], 1);
        assert_eq!(info.kind_core_counts[CoreKind::Performance.index()], 1);
        assert_eq!(info.lps[0].kind, CoreKind::Efficiency);
        assert_eq!(info.lps[1].kind, CoreKind::Performance);
    }

    #[test]
    fn three_efficiency_classes_include_low_power_efficiency() {
        let raw = RawTopology {
            cores: vec![core(0, &[0]), core(1, &[1]), core(2, &[2])],
            packages: vec![vec![0, 1, 2]],
            ..RawTopology::default()
        };
        let info = info(raw);

        assert_eq!(info.lps[0].kind, CoreKind::LpEfficiency);
        assert_eq!(info.lps[1].kind, CoreKind::Efficiency);
        assert_eq!(info.lps[2].kind, CoreKind::Performance);
    }

    #[test]
    fn l3_domains_use_unified_cache_relations_only() {
        let raw = RawTopology {
            cores: vec![core(0, &[0]), core(0, &[1])],
            packages: vec![vec![0, 1]],
            caches: vec![
                cache(CacheLevel::L3, CacheType::Data, 1024, &[0, 1]),
                cache(CacheLevel::L3, CacheType::Unified, 2048, &[0, 1]),
            ],
            ..RawTopology::default()
        };
        let info = info(raw);

        assert_eq!(info.l3_domains.len(), 1);
        assert_eq!(info.l3_domains[0].size_bytes, 2048);
        assert!(info.l3_domains[0].mask.contains(0));
        assert!(info.l3_domains[0].mask.contains(1));
        assert_eq!(info.lps[0].l3_domain, 0);
        assert_eq!(info.lps[1].l3_domain, 0);
    }

    #[test]
    fn sparse_numa_node_ids_count_distinct_nodes() {
        let raw = RawTopology {
            cores: vec![core(0, &[0]), core(0, &[1])],
            packages: vec![vec![0, 1]],
            numa_nodes: vec![(0, vec![0]), (2, vec![1])],
            ..RawTopology::default()
        };
        let info = info(raw);

        assert_eq!(info.numa_node_count, 2);
        assert_eq!(info.lps[0].numa_node, 0);
        assert_eq!(info.lps[1].numa_node, 2);
    }
}
