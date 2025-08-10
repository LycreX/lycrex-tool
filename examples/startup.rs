use lycrex_tool::system;

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    #[cfg(target_os = "macos")]
    test_macos_startup_management()?;
    
    Ok(())
}

#[cfg(target_os = "macos")]
fn test_macos_startup_management() -> Result<(), Box<dyn std::error::Error>> {
    use system::startup::{StartupManager, StartupType};    
    let manager = StartupManager;
    
    println!("\n1. Get All Startup Entries:");
    match manager.list_all() {
        Ok(entries) => {
            println!("Found {} entries:", entries.len());
            for (i, entry) in entries.iter().enumerate().take(10) {
                println!("  {}. {} ({})", i + 1, entry.name, match entry.startup_type {
                    StartupType::LaunchAgent => "LaunchAgent",
                    StartupType::LaunchDaemon => "LaunchDaemon", 
                    StartupType::LoginItems => "LoginItems",
                    _ => "Other",
                });
                println!("     Command: {}", entry.command);
                println!("     Enabled: {}", if entry.enabled { "Yes" } else { "No" });
                if let Some(desc) = &entry.description {
                    println!("     Description: {}", desc);
                }
                println!();
            }
            if entries.len() > 10 {
                println!("  ... {} entries not displayed", entries.len() - 10);
            }
        }
        Err(e) => println!("Get startup entries failed: {}", e),
    }
    
    println!("\n2. Supported Startup Types:");
    let supported_types = manager.get_supported_types();
    for startup_type in supported_types {
        println!("  - {:?}", startup_type);
    }
    
    println!("\n3. Startup Management Features Implemented:");
    println!("  - LaunchAgent supported (user-level startup items)");
    println!("  - LaunchDaemon supported (system-level startup items, requires root privileges)");  
    println!("  - LoginItems supported (login items)");
    println!("  - Full plist file parsing and generation");
    println!("  - launchctl command integration");
    println!("  - osascript login item management");
    
    /*
    let test_entry = StartupEntry::new(
        "Test Application".to_string(),
        "/usr/bin/echo".to_string(),
        StartupType::LaunchAgent,
    ).with_arguments(vec!["Hello from startup!".to_string()]);
    
    match manager.add_entry(&test_entry) {
        Ok(_) => println!("Successfully added test startup entry"),
        Err(e) => println!("Failed to add startup entry: {}", e),
    }
    */
    
    Ok(())
} 