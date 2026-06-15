//! Shared `expected.txt` checker for fixture-driven detection tests.
//!
//! Each fixture carries an `expected.txt` of flat `key=value` assertions
//! against the produced [`CpuInfo`]. The format is deliberately
//! language-neutral: the same external fixture corpus drives Rust and Zig
//! conformance tests. Linux fixtures are recorded sysfs trees; macOS fixtures
//! are recorded sysctl dumps. Both feed this one checker.
//!
//! Identity keys (vendor/model/features) are never asserted: on x86_64 they
//! come from live cpuid regardless of the fixture tree, and the macOS feature
//! flags only exist on aarch64 builds.

use std::path::{Path, PathBuf};

use crate::platform::ranges::parse_range_list_str;
use crate::{CoreKind, CpuInfo, Lp};

// TODO(aljen): Remove hardcoded candidates
fn fixture_base() -> PathBuf {
    if let Some(path) = std::env::var_os("GDT_CPUS_FIXTURES") {
        return PathBuf::from(path);
    }

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        // Normal Rust-repo checkout: submodule/data repo at repo-root/testdata.
        manifest.join("../../testdata/gdt-cpus/fixtures"),
        // gdt monorepo/dev checkout: Rust repo nested under temp/gdt-cpus-rs.
        manifest.join("../../../../testdata/gdt-cpus/fixtures"),
        // Backward-compatible local corpus, if someone has not moved it yet.
        manifest.join("fixtures"),
    ];

    candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}

pub(crate) fn fixture_root(name: &str) -> PathBuf {
    fixture_base().join(name)
}

fn kind_by_name(name: &str) -> CoreKind {
    match name {
        "performance" => CoreKind::Performance,
        "efficiency" => CoreKind::Efficiency,
        "lp_efficiency" => CoreKind::LpEfficiency,
        other => panic!("unknown kind name in expected.txt: {}", other),
    }
}

/// Universal invariants every detection result must satisfy, fixture or live.
pub(crate) fn assert_invariants(info: &CpuInfo) {
    let smt0 = info.lps.iter().filter(|lp| lp.smt_index == 0).count();
    assert_eq!(smt0, info.core_count as usize, "one smt_index==0 per core");

    let kind_sum: u16 = info.kind_core_counts.iter().sum();
    assert_eq!(kind_sum, info.core_count, "kind counts partition cores");

    assert!(
        !info.performance_core_mask().is_empty(),
        "performance mask never empty (homogeneous => all performance)"
    );

    // L3 domains disjoint + covering all LPs that report one.
    for (i, a) in info.l3_domains.iter().enumerate() {
        for b in info.l3_domains.iter().skip(i + 1) {
            assert!(
                a.mask.intersection(&b.mask).is_empty(),
                "L3 domains must be disjoint"
            );
        }
    }

    for lp in &info.lps {
        if lp.l3_domain != Lp::NO_L3 {
            assert!(
                info.l3_domains[lp.l3_domain as usize]
                    .mask
                    .contains(lp.os_id as usize),
                "lp {} not covered by its own L3 domain",
                lp.os_id
            );
        }
    }
}

fn lp_by_os_id(info: &CpuInfo, os_id: usize) -> &Lp {
    info.lps
        .iter()
        .find(|lp| lp.os_id as usize == os_id)
        .unwrap_or_else(|| panic!("expected.txt references unknown lp {}", os_id))
}

/// Checks every `expected.txt` line of fixture `name` against `info`,
/// after asserting the universal invariants.
pub(crate) fn check_expected(info: &CpuInfo, name: &str) {
    assert_invariants(info);

    let expected = std::fs::read_to_string(fixture_root(name).join("expected.txt"))
        .unwrap_or_else(|e| panic!("missing expected.txt for {}: {}", name, e));

    for line in expected.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("malformed expected line: {}", line));

        check_key(info, key, value, name);
    }
}

fn check_key(info: &CpuInfo, key: &str, value: &str, fixture: &str) {
    let parts: Vec<&str> = key.split('.').collect();
    let actual: String = match parts.as_slice() {
        ["lp_count"] => info.lps.len().to_string(),
        ["core_count"] => info.core_count.to_string(),
        ["socket_count"] => info.socket_count.to_string(),
        ["numa_node_count"] => info.numa_node_count.to_string(),
        ["l3_domain_count"] => info.l3_domains.len().to_string(),
        ["kind", kind] => info.kind_core_counts[kind_by_name(kind).index()].to_string(),
        ["l3", n, field] => {
            let d = &info.l3_domains[n.parse::<usize>().unwrap()];

            match *field {
                "size_bytes" => d.size_bytes.to_string(),
                "core_count" => d.core_count.to_string(),
                "lps" => {
                    let want = parse_range_list_str(value).unwrap();

                    for id in &want {
                        assert!(
                            d.mask.contains(*id),
                            "{}: l3.{}.lps missing {}",
                            fixture,
                            n,
                            id
                        );
                    }

                    assert_eq!(
                        d.mask.count(),
                        want.len(),
                        "{}: l3.{}.lps cardinality",
                        fixture,
                        n
                    );

                    return;
                }
                other => panic!("unknown l3 field: {}", other),
            }
        }
        [cache @ ("l1d" | "l1i" | "l2"), kind, field] => {
            let table = match *cache {
                "l1d" => &info.l1d,
                "l1i" => &info.l1i,
                _ => &info.l2,
            };
            let ci = &table[kind_by_name(kind).index()];

            match *field {
                "size_bytes" => ci.size_bytes.to_string(),
                "line_bytes" => ci.line_bytes.to_string(),
                "shared_by" => ci.shared_by.to_string(),
                other => panic!("unknown cache field: {}", other),
            }
        }
        ["lp", n, field] => {
            let lp = lp_by_os_id(info, n.parse::<usize>().unwrap());

            match *field {
                "core" => lp.core.to_string(),
                "socket" => lp.socket.to_string(),
                "smt_index" => lp.smt_index.to_string(),
                "numa_node" => lp.numa_node.to_string(),
                "perf_hint" => lp.perf_hint.to_string(),
                "cpu_part" => lp.cpu_part.to_string(),
                "kind" => lp
                    .kind
                    .to_string()
                    .to_lowercase()
                    .replace("lpefficiency", "lp_efficiency"),
                "l3_domain" => {
                    if lp.l3_domain == Lp::NO_L3 {
                        "none".to_string()
                    } else {
                        lp.l3_domain.to_string()
                    }
                }
                other => panic!("unknown lp field: {}", other),
            }
        }
        _ => panic!("unknown expected.txt key: {}", key),
    };

    assert_eq!(actual, value, "{}: key {}", fixture, key);
}
