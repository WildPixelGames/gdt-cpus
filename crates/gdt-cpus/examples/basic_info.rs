//! Example that displays basic CPU information.

use gdt_cpus::{CoreKind, CpuInfo};

fn main() {
    let info = match CpuInfo::detect() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error retrieving CPU information: {}", e);
            return;
        }
    };

    println!("CPU Information:");
    println!("---------------");
    println!("Vendor: {}", info.vendor);
    println!("Model: {}", info.model_name);
    println!("Sockets: {}", info.socket_count);
    println!("Physical cores: {}", info.num_physical_cores());
    println!("Logical cores: {}", info.num_logical_cores());
    println!("Performance cores: {}", info.num_performance_cores());
    println!("Efficiency cores: {}", info.num_efficiency_cores());
    println!("LP-Efficiency cores: {}", info.num_lp_efficiency_cores());
    println!("NUMA nodes: {}", info.numa_node_count);
    println!(
        "Hybrid architecture: {}",
        if info.is_hybrid() { "Yes" } else { "No" }
    );

    // Column widths sized to the largest domain index so the index/`-domain`
    // columns stay aligned on big machines (a 128-core part has 128 L2 domains).
    let digits = |n: usize| n.max(1).to_string().len();
    let l3_w = digits(info.l3_domains.len().saturating_sub(1));
    let l2_w = digits(info.l2_domains.len().saturating_sub(1));

    println!("\nL3 domains: {}", info.l3_domains.len());
    for (i, d) in info.l3_domains.iter().enumerate() {
        println!(
            "  domain {:>lw$}: {} MiB, {} cores, {} threads, lps {}",
            i,
            d.size_bytes / (1024 * 1024),
            d.core_count,
            d.mask.count(),
            d.mask,
            lw = l3_w,
        );
    }

    println!("\nL2 domains: {}", info.l2_domains.len());
    for (i, d) in info.l2_domains.iter().enumerate() {
        let l3 = if d.l3_domain == gdt_cpus::Lp::NO_L3 {
            "-".to_string()
        } else {
            d.l3_domain.to_string()
        };
        println!(
            "  domain {:>w$}: {} KB, {} cores, {} threads, l3-domain {:>lw$}, lps {}",
            i,
            d.size_bytes / 1024,
            d.core_count,
            d.mask.count(),
            l3,
            d.mask,
            w = l2_w,
            lw = l3_w,
        );
    }

    println!("\nPer-kind caches:");
    for kind in [
        CoreKind::Performance,
        CoreKind::Efficiency,
        CoreKind::LpEfficiency,
    ] {
        let k = kind.index();
        if info.kind_core_counts[k] == 0 {
            continue;
        }
        println!(
            "  {}: L1d {} KB / L1i {} KB / L2 {} KB (L2 shared by {} threads)",
            kind,
            info.l1d[k].size_bytes / 1024,
            info.l1i[k].size_bytes / 1024,
            info.l2[k].size_bytes / 1024,
            info.l2[k].shared_by,
        );
    }

    println!("\nLogical processors:");
    for lp in &info.lps {
        println!(
            "  lp {:>3}: core {:>3} smt {} socket {} l3-domain {:>lw$} l2-domain {:>w$} numa {} perf {:>4} kind {}",
            lp.os_id,
            lp.core,
            lp.smt_index,
            lp.socket,
            if lp.l3_domain == gdt_cpus::Lp::NO_L3 {
                "-".to_string()
            } else {
                lp.l3_domain.to_string()
            },
            if lp.l2_domain == gdt_cpus::Lp::NO_L2 {
                "-".to_string()
            } else {
                lp.l2_domain.to_string()
            },
            lp.numa_node,
            lp.perf_hint,
            lp.kind,
            lw = l3_w,
            w = l2_w,
        );
    }

    println!("\nCPU Features:");
    println!(
        "  {}",
        info.features
            .iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(", ")
    );
}
