pub fn main() {
    use lycrex_tool::lycrex::logger::start_log_simple;
    use lycrex_tool::{info, error};
    start_log_simple("debug", false, 6).expect("Init log failed");
    info!("example", "Init log success");


    #[cfg(target_os = "windows")]
    {
        use lycrex_tool::memory::win_memory::ProcessInstance;
        use lycrex_tool::memory::win_memory::to_u32;
        use lycrex_tool::lycrex::logo;

        logo::display_logo(Some(2));

        let proc = match ProcessInstance::new_by_name("tes.exe") {
            Ok(proc) => proc,
            Err(e) => {
                error!("example", "Create process instance failed: {}", e);
                return;
            }
        };

        loop {
            use std::time::Duration;
            use std::thread;

            thread::sleep(Duration::from_secs(1));
            proc.write_u32(0x23000, 666).expect("Write memory failed");
            let bytes = proc.read_memory(0x23000, 16).expect("Read memory failed");
            let val = to_u32(&bytes);
            info!("example", "val: {}", val);
        }
    }

    #[cfg(target_os = "macos")]
    {
        use lycrex_tool::lycrex::logo;
        use lycrex_tool::system::memory::MemoryManager;


        logo::display_logo(Some(2));

        let memory = MemoryManager::new();
        let proc = memory.create_process_instance_by_pid(48180).expect("Create process instance failed");
        loop {
            use std::time::Duration;
            use std::thread;
            
            let data_seg_offset: usize = 0x54000;
            let health_offset: usize = 0x0;
            let mana_offset: usize = 0x4;
            let coins_offset: usize = 0x8;
            let level_offset: usize = 0xC;
            
            match proc.read_u32(data_seg_offset + health_offset) {
                Ok(val) => {
                    info!("example", "health: {}", val);
                }
                Err(e) => {
                    error!("example", "Read memory failed: {}", e);
                }
            }
            match proc.read_u32(data_seg_offset + mana_offset) {
                Ok(val) => {
                    info!("example", "mana: {}", val);
                }
                Err(e) => {
                    error!("example", "Read memory failed: {}", e);
                }
            }
            match proc.read_u32(data_seg_offset + coins_offset) {
                Ok(val) => {
                    info!("example", "coins: {}", val);
                }
                Err(e) => {
                    error!("example", "Read memory failed: {}", e);
                }
            }
            match proc.read_u32(data_seg_offset + level_offset) {
                Ok(val) => {
                    info!("example", "level: {}\n", val);
                }
                Err(e) => {
                    error!("example", "Read memory failed: {}", e);
                }
            }

            thread::sleep(Duration::from_secs(1));

        }
    }
}