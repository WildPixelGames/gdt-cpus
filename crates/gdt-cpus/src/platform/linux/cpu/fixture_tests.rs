//! Fixture-driven Linux detection tests.
//!
//! Each fixture is a recorded or synthetic `/sys` (+ minimal `/proc`) tree
//! plus an `expected.txt` of flat `key=value` assertions - the checker and
//! the format contract live in `crate::platform::fixture_expected`.

use super::detect_at;
use crate::platform::fixture_expected::{check_expected, fixture_root};

/// Runs detection against a fixture tree and checks every `expected.txt` line.
fn run_fixture(name: &str) {
    let root = fixture_root(name);

    // NOTE: the recorded corpus is external to the crate. Plain `cargo test`
    // without the data checkout skips; CI should set GDT_CPUS_FIXTURES or fetch
    // the shared testdata tree.
    if !root.exists() {
        eprintln!(
            "fixture {} not present at {} (set GDT_CPUS_FIXTURES to enable) - skipped",
            name,
            root.display()
        );
        return;
    }

    let info = detect_at(&root.join("sys"), &root.join("proc"))
        .unwrap_or_else(|e| panic!("detect_at failed for fixture {}: {}", name, e));

    check_expected(&info, name);
}

#[test]
fn fixture_5950x_two_l3_domains() {
    run_fixture("sysfs-5950x");
}

#[test]
fn fixture_wsl2_virtualized_l3() {
    run_fixture("sysfs-wsl2");
}

#[test]
fn fixture_hybrid_x86_core_type_chain() {
    run_fixture("sysfs-hybrid-x86");
}

#[test]
fn fixture_biglittle_arm_capacity_thresholds() {
    run_fixture("sysfs-biglittle-arm");
}

#[test]
fn fixture_i7_6700_monolithic_l3() {
    run_fixture("sysfs-i7-6700");
}

#[test]
fn fixture_cix_p1_three_tier_capacity() {
    run_fixture("sysfs-cix-p1");
}

#[test]
fn fixture_pi5_homogeneous_degenerate_numa() {
    run_fixture("sysfs-pi5");
}

#[test]
fn fixture_numa2_disjoint_nodes_survive() {
    run_fixture("sysfs-numa2");
}

#[test]
fn fixture_numa_sparse_node_ids() {
    // Sparse node ids (node1 absent): guards the break-on-first-gap bug that
    // counted only contiguous nodes and stranded later-node LPs on node 0.
    run_fixture("sysfs-numa-sparse");
}
