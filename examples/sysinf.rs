use lycrex_tool::system::SystemTools;

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    //  SystemTools
    let sysinfo = SystemTools::sysinfo();

    println!("System Info:");
    match sysinfo.get_basic_info() {
        Ok(info) => {
            println!("  System: {} {}", info.os_name, info.os_version);
            println!("  Kernel: {}", info.kernel_version);
            println!("  Arch: {}", info.arch);
            println!("  Hostname: {}", info.hostname);
            println!("  Username: {}", info.username);
            println!("  Uptime: {} s", info.uptime);
        }
        Err(e) => println!("  Get System Info Error: {}", e),
    }
    
    println!();
    
    println!("Memory Info:");
    match sysinfo.get_memory_info() {
        Ok(mem) => {
            println!("  Total Memory: {}", sysinfo.format_memory_size(mem.total));
            println!("  Used: {}", sysinfo.format_memory_size(mem.used));
            println!("  Available: {}", sysinfo.format_memory_size(mem.available));
            println!("  Usage: {:.1}%", mem.usage_percent);
            println!("  Swap Total: {}", sysinfo.format_memory_size(mem.swap_total));
            println!("  Swap Used: {}", sysinfo.format_memory_size(mem.swap_used));
        }
        Err(e) => println!("  Get Memory Info Error: {}", e),
    }
    
    println!();
    
    println!("CPU Info:");
    match sysinfo.get_cpu_info() {
        Ok(cpus) => {
            if let Some(first_cpu) = cpus.first() {
                println!("  CPU Model: {}", first_cpu.name);
                println!("  CPU Brand: {}", first_cpu.brand);
                println!("  Frequency: {} MHz", first_cpu.frequency);
                println!("  Vendor ID: {}", first_cpu.vendor_id);
                println!("  CPU Core Count: {}", cpus.len());
            }
            
            println!("  Each Core Usage:");
            for (i, cpu) in cpus.iter().take(5).enumerate() {
                println!("    Core {}: {:.1}%", i, cpu.usage);
            }
            if cpus.len() > 8 {
                println!("    ... {} more cores", cpus.len() - 8);
            }
        }
        Err(e) => println!("  Get CPU Info Error: {}", e),
    }
    
    println!();
    
    println!("Disk Info:");
    match sysinfo.get_disk_info() {
        Ok(disks) => {
            for disk in disks.iter().take(99) {
                let usage_percent = if disk.total_space > 0 {
                    (disk.used_space as f64 / disk.total_space as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("  {} {}", disk.mount_point, disk.name);
                println!("  Filesystem: {}", disk.filesystem);
                println!("  Total Space: {}", sysinfo.format_memory_size(disk.total_space));
                println!("  Used: {}", sysinfo.format_memory_size(disk.used_space));
                println!("  Available: {}", sysinfo.format_memory_size(disk.available_space));
                println!("  Usage: {:.1}%", usage_percent);
                println!("  Removable: {}", if disk.is_removable { "Yes" } else { "No" });
                println!();
            }
        }
        Err(e) => println!("  Get Disk Info Error: {}", e),
    }
    
    println!("Network Interface Info:");
    match sysinfo.get_network_info() {
        Ok(networks) => {
            for net in networks.iter().take(99  ) {
                println!("  {} {}", net.name, net.mac_address);
                println!();
            }
        }
        Err(e) => println!("  Get Network Info Error: {}", e),
    }

    println!("Permission Check:");
    if sysinfo.has_admin_privileges() {
        println!("  ✅ Current has Admin Privileges");
    } else {
        println!("  ⚠️ Current has no Admin Privileges");
    }
    
    println!("\n=== Demo Completed ===");
    Ok(())
} 