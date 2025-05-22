//! Example that displays basic CPU information.

fn main() {
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .init();

    // Get CPU information
    let info = match gdt_cpus::cpu_info() {
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
    println!("Physical cores: {}", info.total_physical_cores);
    println!("Logical cores: {}", info.total_logical_processors);
    println!("Performance cores: {}", info.total_performance_cores);
    println!("Efficiency cores: {}", info.total_efficiency_cores);
    println!(
        "Hybrid architecture: {}",
        if info.is_hybrid() { "Yes" } else { "No" }
    );

    // Print more detailed information about each processor/socket
    for (i, socket) in info.sockets.iter().enumerate() {
        println!("\nProcessor #{} (Socket ID: {})", i, socket.id);

        // Print cache information if available
        if let Some(ref l3) = socket.l3_cache {
            println!("  L3 Cache: {} KB", l3.size_bytes / 1024);
        }

        // Print information about each core
        println!("  Cores:");
        for core in &socket.cores {
            println!(
                "    Core #{}: {} core with {} threads",
                core.id,
                core.core_type,
                core.logical_processor_ids.len()
            );

            // Print cache information if available
            if let Some(ref l1i) = core.l1_instruction_cache {
                println!("      L1i Cache: {} KB", l1i.size_bytes / 1024);
            }
            if let Some(ref l1d) = core.l1_data_cache {
                println!("      L1d Cache: {} KB", l1d.size_bytes / 1024);
            }
            if let Some(ref l2) = core.l2_cache {
                println!("      L2 Cache: {} KB", l2.size_bytes / 1024);
            }
        }
    }

    // Print CPU features
    println!("\nCPU Features:",);
    println!(
        "  {}",
        info.features
            .iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(", ")
    );
}
