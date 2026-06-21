//! macOS CPU topology detection via `sysctl` - flat model, synthetic LP layout.
//!
//! Apple Silicon only; see the compile_error gate in `mod.rs`.
//!
//! macOS exposes no LP -> core mapping and no thread affinity, so the per-LP
//! records are SYNTHETIC with a PINNED layout: kinds in perflevel order
//! (P first, then E), core-major within a kind - core `i` of a kind with SMT
//! ratio `r` gets LP ids `[base + i*r, base + i*r + r)` with
//! `smt_index = 0..r-1` (r = 1 on all Apple Silicon to date; the math stays
//! general). The ids give masks/counts a consistent shape; they are NOT OS
//! thread-placement ids - there is nothing to place against.
//!
//! Caches map 1:1 onto the per-kind model (`hw.perflevelN.*`). L3 domains are
//! synthesized from `hw.perflevelN.{l3cachesize,cpusperl3}` when present -
//! every current Apple Silicon chip reports neither (the SLC is not exposed)
//! ⇒ zero domains in practice, but the interface defines the keys
//! (Optimization Guide, Appendix B.2), so we query instead of hardcoding.
//! Every key is defaulted on absence - no panic paths.

use crate::{
    AffinityMask, CacheInfo, CoreKind, CpuFeatures, CpuInfo, Error, L2Domain, L3Domain, Lp, Result,
    Vendor,
};

/// The detection pipeline's read seam.
///
/// The live implementation wraps `sysctlbyname`; the fixture implementation
/// (tests) replays a recorded dump - which is how non-macOS CI exercises this
/// pipeline at all. `None` = key absent (ENOENT) or not of the requested
/// shape; every caller defaults on absence.
pub(crate) trait SysctlSource {
    /// Integer key, width-resolved (4- and 8-byte kernel values both arrive
    /// zero-extended to u64).
    fn int(&self, key: &str) -> Option<u64>;
    /// String key.
    fn string(&self, key: &str) -> Option<String>;
}

/// Live `sysctlbyname`-backed source.
#[cfg(target_os = "macos")]
pub(crate) struct LiveSysctl;

#[cfg(target_os = "macos")]
impl SysctlSource for LiveSysctl {
    fn int(&self, key: &str) -> Option<u64> {
        super::utils::sysctlbyname_int::<u64>(key).ok()
    }

    fn string(&self, key: &str) -> Option<String> {
        super::utils::sysctlbyname_string(key).ok()
    }
}

/// Detects CPU vendor, model name, and features using `sysctl`.
fn detect_cpu_via_sysctl(
    src: &impl SysctlSource,
    vendor: &mut Vendor,
    model_name: &mut String,
    features: &mut CpuFeatures,
) {
    *model_name = src
        .string("machdep.cpu.brand_string")
        .unwrap_or_else(|| "Unknown".to_string());

    let vendor_str = src
        .string("machdep.cpu.vendor")
        .unwrap_or_else(|| "Unknown".to_string());

    // Apple Silicon only: vendor is Apple unless sysctl reports something
    // unrecognizable (then Other - never guess Intel/AMD on this platform).
    *vendor = if vendor_str.eq_ignore_ascii_case("apple")
        || model_name.to_lowercase().contains("apple")
    {
        Vendor::Apple
    } else {
        Vendor::Other
    };

    *features = CpuFeatures::default();
    // NOTE: CpuFeatures flags are per-ARCH (NEON/AES/SHA/CRC32 exist only on
    // aarch64 builds), so feature mapping is compiled out of the x86_64 CI
    // runs of this pipeline - fixtures there cover topology/caches/kinds,
    // feature mapping is covered on aarch64 hosts.
    #[cfg(target_arch = "aarch64")]
    {
        let flag = |key: &str| src.int(key).map(|v| v == 1).unwrap_or(false);

        // Standardized names first, legacy fallbacks second (Apple Silicon CPU
        // Optimization Guide, Appendix B.1: "neon" et al. are legacy aliases).
        if flag("hw.optional.AdvSIMD") || flag("hw.optional.neon") {
            features.insert(CpuFeatures::NEON);
        }
        if flag("hw.optional.arm.FEAT_AES") {
            features.insert(CpuFeatures::AES);
        }
        if flag("hw.optional.arm.FEAT_SHA1")
            || flag("hw.optional.arm.FEAT_SHA3")
            || flag("hw.optional.arm.FEAT_SHA256")
            || flag("hw.optional.arm.FEAT_SHA512")
        {
            features.insert(CpuFeatures::SHA);
        }
        if flag("hw.optional.arm.FEAT_CRC32") || flag("hw.optional.armv8_crc32") {
            features.insert(CpuFeatures::CRC32);
        }
        if flag("hw.optional.arm.FEAT_FP16") {
            features.insert(CpuFeatures::FP16);
        }
        if flag("hw.optional.arm.FEAT_DotProd") {
            features.insert(CpuFeatures::DOTPROD);
        }
        if flag("hw.optional.arm.FEAT_I8MM") {
            features.insert(CpuFeatures::I8MM);
        }
        if flag("hw.optional.arm.FEAT_BF16") {
            features.insert(CpuFeatures::BF16);
        }
        // SVE2 is queried for symmetry; Apple Silicon implements no SVE, so this
        // is always absent there (the flag exists for non-Apple ARM detection).
        if flag("hw.optional.arm.FEAT_SVE2") {
            features.insert(CpuFeatures::SVE2);
        }
        if flag("hw.optional.arm.FEAT_LSE") {
            features.insert(CpuFeatures::LSE);
        }
        if flag("hw.optional.arm.FEAT_JSCVT") {
            features.insert(CpuFeatures::JSCVT);
        }
        if flag("hw.optional.arm.FEAT_LRCPC") {
            features.insert(CpuFeatures::LRCPC);
        }
        if flag("hw.optional.arm.FEAT_PMULL") {
            features.insert(CpuFeatures::PMULL);
        }
        if flag("hw.optional.arm.FEAT_RDM") {
            features.insert(CpuFeatures::RDM);
        }
        if flag("hw.optional.arm.FEAT_FHM") || flag("hw.optional.armv8_2_fhm") {
            features.insert(CpuFeatures::FHM);
        }
        if flag("hw.optional.arm.FEAT_FCMA") {
            features.insert(CpuFeatures::FCMA);
        }
        if flag("hw.optional.arm.FEAT_LSE2") {
            features.insert(CpuFeatures::LSE2);
        }
        if flag("hw.optional.arm.FEAT_LRCPC2") {
            features.insert(CpuFeatures::LRCPC2);
        }
        if flag("hw.optional.arm.FEAT_SM3") {
            features.insert(CpuFeatures::SM3);
        }
        if flag("hw.optional.arm.FEAT_SM4") {
            features.insert(CpuFeatures::SM4);
        }
        if flag("hw.optional.arm.FEAT_SVE_AES") {
            features.insert(CpuFeatures::SVEAES);
        }
        if flag("hw.optional.arm.FEAT_SVE_PMULL128") {
            features.insert(CpuFeatures::SVEPMULL);
        }
        if flag("hw.optional.arm.FEAT_SVE_BitPerm") {
            features.insert(CpuFeatures::SVEBITPERM);
        }
        if flag("hw.optional.arm.FEAT_SVE_SHA3") {
            features.insert(CpuFeatures::SVESHA3);
        }
        if flag("hw.optional.arm.FEAT_SVE_SM4") {
            features.insert(CpuFeatures::SVESM4);
        }
        if flag("hw.optional.arm.FEAT_SVE_I8MM") {
            features.insert(CpuFeatures::SVEI8MM);
        }
        if flag("hw.optional.arm.FEAT_SVE_BF16") {
            features.insert(CpuFeatures::SVEBF16);
        }
    }
}

/// One per-kind group of cores, read from `hw.perflevelN.*`.
struct KindGroup {
    kind: CoreKind,
    cores: usize,
    lps: usize,
    l1d: CacheInfo,
    l1i: CacheInfo,
    l2: CacheInfo,
    /// `hw.perflevelN.l3cachesize` - 0 (ENOENT) on every current Apple Silicon
    /// chip, but the sysctl interface defines it (Optimization Guide, B.2);
    /// read opportunistically instead of hardcoding "no L3".
    l3_size: u64,
    /// `hw.perflevelN.cpusperl3` - cores sharing one L3 instance.
    cpus_per_l3: usize,
    /// `hw.perflevelN.cpusperl2` - cores sharing one L2 instance.
    cpus_per_l2: usize,
}

fn read_perflevel(
    src: &impl SysctlSource,
    level: u32,
    kind: CoreKind,
    line: u16,
) -> Option<KindGroup> {
    let cores = src
        .int(&format!("hw.perflevel{}.physicalcpu", level))
        .unwrap_or(0);

    if cores == 0 {
        return None;
    }

    let lps = src
        .int(&format!("hw.perflevel{}.logicalcpu", level))
        .unwrap_or(cores) as usize;
    let smt = (lps / cores as usize).max(1) as u16;
    let cache = |key: &str| -> u64 {
        src.int(&format!("hw.perflevel{}.{}", level, key))
            .unwrap_or(0)
    };
    let cpus_per_l2 = cache("cpusperl2").max(1) as u16;
    let l3_size = cache("l3cachesize");

    // Absent cpusperl3 with a present L3 = one domain spanning the group.
    let cpus_per_l3 = match cache("cpusperl3") as usize {
        0 => cores as usize,
        n => n,
    };

    Some(KindGroup {
        kind,
        cores: cores as usize,
        lps,
        l1d: CacheInfo {
            size_bytes: cache("l1dcachesize"),
            line_bytes: line,
            shared_by: smt,
        },
        l1i: CacheInfo {
            size_bytes: cache("l1icachesize"),
            line_bytes: line,
            shared_by: smt,
        },
        l2: CacheInfo {
            size_bytes: cache("l2cachesize"),
            line_bytes: line,
            shared_by: cpus_per_l2 * smt,
        },
        l3_size,
        cpus_per_l3,
        cpus_per_l2: cpus_per_l2 as usize,
    })
}

/// Detects CPU information on macOS (live sysctl).
#[cfg(target_os = "macos")]
pub fn detect_cpu_info() -> Result<CpuInfo> {
    detect_at(&LiveSysctl)
}

/// The detection pipeline against any [`SysctlSource`] - pure logic, compiled
/// (and fixture-tested) on every platform.
pub(crate) fn detect_at(src: &impl SysctlSource) -> Result<CpuInfo> {
    // --- Identity ---
    let mut vendor = Vendor::Unknown;
    let mut model_name = "Unknown".to_string();
    let mut features = CpuFeatures::default();

    detect_cpu_via_sysctl(src, &mut vendor, &mut model_name, &mut features);

    // --- Counts ---
    let socket_count = src.int("hw.packages").unwrap_or(1).max(1) as usize;
    let physical = src.int("hw.physicalcpu").unwrap_or(0) as usize;
    let logical = src.int("hw.logicalcpu").unwrap_or(0) as usize;

    if physical == 0 || logical == 0 {
        return Err(Error::Detection(
            "hw.physicalcpu / hw.logicalcpu unavailable".to_string(),
        ));
    }

    let line = src.int("hw.cachelinesize").unwrap_or(64) as u16;

    // --- Per-kind groups: perflevel0 = P, perflevel1 = E; absence => homogeneous all-P ---
    let mut groups: Vec<KindGroup> = Vec::new();

    if let Some(p) = read_perflevel(src, 0, CoreKind::Performance, line) {
        groups.push(p);
    }

    if let Some(e) = read_perflevel(src, 1, CoreKind::Efficiency, line) {
        groups.push(e);
    }

    if groups.is_empty() || groups[0].kind != CoreKind::Performance {
        // No perflevels or only an E group reported (anomaly - every Apple
        // Silicon macOS has hw.perflevel0): homogeneous invariant says all-P.
        let smt = (logical / physical).max(1) as u16;
        let direct = |key: &str| -> u64 { src.int(key).unwrap_or(0) };

        groups = vec![KindGroup {
            kind: CoreKind::Performance,
            cores: physical,
            lps: logical,
            l1d: CacheInfo {
                size_bytes: direct("hw.l1dcachesize"),
                line_bytes: line,
                shared_by: smt,
            },
            l1i: CacheInfo {
                size_bytes: direct("hw.l1icachesize"),
                line_bytes: line,
                shared_by: smt,
            },
            l2: CacheInfo {
                size_bytes: direct("hw.l2cachesize"),
                line_bytes: line,
                shared_by: smt,
            },
            l3_size: direct("hw.l3cachesize"),
            cpus_per_l3: physical,
            cpus_per_l2: physical,
        }];
    }

    // --- Synthetic LP records (pinned layout, see module doc) ---
    // L3: no current Apple Silicon exposes one (the SLC is hidden ⇒ in
    // practice zero domains, every LP keeps Lp::NO_L3) - but the sysctl
    // interface DEFINES hw.perflevelN.{l3cachesize,cpusperl3} (Optimization
    // Guide, Appendix B.2), so synthesize domains from them when present
    // instead of hardcoding the absence: consecutive cores within a kind,
    // cpus_per_l3 per domain.
    let mut lps: Vec<Lp> = Vec::with_capacity(logical);
    let mut l3_domains: Vec<L3Domain> = Vec::new();
    let mut l2_domains: Vec<L2Domain> = Vec::new();
    let mut l1d = [CacheInfo::default(); CoreKind::COUNT];
    let mut l1i = [CacheInfo::default(); CoreKind::COUNT];
    let mut l2 = [CacheInfo::default(); CoreKind::COUNT];
    let mut kind_core_counts = [0u16; CoreKind::COUNT];
    let mut next_lp: usize = 0;
    let mut next_core: usize = 0;

    let total_cores: usize = groups.iter().map(|g| g.cores).sum();

    for (group_idx, group) in groups.iter().enumerate() {
        // perf_hint: perflevel order, higher = faster (perflevel0 = best).
        // Coarse but honest - macOS exposes no finer per-core signal.
        let perf_hint = (groups.len() - group_idx) as u16;
        let k = group.kind.index();

        l1d[k] = group.l1d;
        l1i[k] = group.l1i;
        l2[k] = group.l2;
        kind_core_counts[k] += group.cores as u16;

        let smt = (group.lps / group.cores).max(1);
        let group_domain_base = l3_domains.len();
        let group_l2_base = l2_domains.len();

        for core_i in 0..group.cores {
            // hw.packages is 1 on all Apple Silicon; the even split is kept as
            // a defensive generality, not a supported configuration.
            let socket = (next_core * socket_count / total_cores.max(1)) as u8;
            let l3_domain = if group.l3_size > 0 && l3_domains.len() < usize::from(Lp::NO_L3) {
                let idx = group_domain_base + core_i / group.cpus_per_l3;

                if idx == l3_domains.len() {
                    l3_domains.push(L3Domain {
                        size_bytes: group.l3_size,
                        mask: AffinityMask::empty(),
                        core_count: 0,
                    });
                }

                l3_domains[idx].core_count += 1;

                idx as u8
            } else {
                Lp::NO_L3
            };

            // L2 domains: chunk this group's cores into cpus_per_l2-sized groups.
            let l2_domain = if group.l2.size_bytes > 0 && l2_domains.len() < usize::from(Lp::NO_L2)
            {
                let idx = group_l2_base + core_i / group.cpus_per_l2.max(1);

                if idx == l2_domains.len() {
                    l2_domains.push(L2Domain {
                        size_bytes: group.l2.size_bytes,
                        mask: AffinityMask::empty(),
                        core_count: 0,
                        l3_domain,
                    });
                }

                l2_domains[idx].core_count += 1;

                idx as u16
            } else {
                Lp::NO_L2
            };

            for sibling in 0..smt {
                if l3_domain != Lp::NO_L3 {
                    l3_domains[l3_domain as usize].mask.add(next_lp);
                }

                if l2_domain != Lp::NO_L2 {
                    l2_domains[l2_domain as usize].mask.add(next_lp);
                }

                lps.push(Lp {
                    os_id: next_lp as u16,
                    core: next_core as u16,
                    socket,
                    l3_domain,
                    l2_domain,
                    numa_node: 0,
                    kind: group.kind,
                    smt_index: sibling as u8,
                    perf_hint,
                    // NOTE(macos): sysctl exposes no per-core MIDR part on Apple
                    // Silicon; perflevel order already classifies P/E. Leave 0.
                    cpu_part: 0,
                });

                next_lp += 1;
            }

            next_core += 1;
        }
    }

    let mut info = CpuInfo {
        lps,
        core_count: next_core as u16,
        socket_count: socket_count as u8,
        numa_node_count: 1,
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

/// Fixture-driven macOS detection tests - these run on EVERY platform, which
/// is the whole point of the [`SysctlSource`] seam: Linux CI catches macOS
/// detection breakage without Apple hardware.
#[cfg(test)]
mod fixture_tests {
    use std::collections::HashMap;

    use super::{SysctlSource, detect_at};
    use crate::platform::fixture_expected::{check_expected, fixture_root};

    /// Replays a recorded `sysctl.txt` dump from the shared fixture corpus.
    ///
    /// Line format, language-neutral on purpose (the Zig test suite parses
    /// the same dumps): `i4 <key> <value>` / `i8 <key> <value>` for integers
    /// (recorded kernel width - Darwin sysctl keys are MIXED-width),
    /// `s <key> <value...>` for strings. `#` comments and blanks ignored.
    struct FixtureSysctl {
        ints: HashMap<String, u64>,
        strs: HashMap<String, String>,
    }

    impl FixtureSysctl {
        fn load(name: &str) -> Self {
            let path = fixture_root(name).join("sysctl.txt");
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing {}: {}", path.display(), e));

            let mut ints = HashMap::new();
            let mut strs = HashMap::new();

            for line in text.lines() {
                let line = line.trim();

                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let mut parts = line.splitn(3, ' ');
                let (tag, key, value) = (
                    parts.next().unwrap(),
                    parts
                        .next()
                        .unwrap_or_else(|| panic!("malformed: {}", line)),
                    parts.next().unwrap_or(""),
                );

                match tag {
                    "i4" | "i8" => {
                        let v = value
                            .parse::<i64>()
                            .unwrap_or_else(|_| panic!("bad int in: {}", line));

                        // Mimic the live helper: 4-byte values arrive
                        // zero-extended, not sign-extended.
                        let v = if tag == "i4" {
                            (v as u32) as u64
                        } else {
                            v as u64
                        };

                        ints.insert(key.to_string(), v);
                    }
                    "s" => {
                        strs.insert(key.to_string(), value.to_string());
                    }
                    other => panic!("unknown sysctl.txt tag {} in: {}", other, line),
                }
            }

            FixtureSysctl { ints, strs }
        }
    }

    impl SysctlSource for FixtureSysctl {
        fn int(&self, key: &str) -> Option<u64> {
            self.ints.get(key).copied()
        }

        fn string(&self, key: &str) -> Option<String> {
            self.strs.get(key).cloned()
        }
    }

    fn run_fixture(name: &str) {
        // Same skip contract as the Linux fixtures. Key on the dump itself, so
        // a pre-authored expected.txt without a recording also skips instead of
        // failing.
        let dump = fixture_root(name).join("sysctl.txt");

        if !dump.exists() {
            eprintln!(
                "fixture {} has no sysctl.txt at {} (set GDT_CPUS_FIXTURES to enable) - skipped",
                name,
                dump.display()
            );
            return;
        }

        let src = FixtureSysctl::load(name);
        let info =
            detect_at(&src).unwrap_or_else(|e| panic!("detect_at failed for {}: {}", name, e));

        check_expected(&info, name);
    }

    #[test]
    fn fixture_m3_max_perflevels() {
        run_fixture("sysctl-m3-max");
    }
}
