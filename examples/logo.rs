pub fn main() {
    #[cfg(all(feature = "win-memory", target_os = "windows"))]
    {
        use lycrex_tool::memory::win_memory::ProcessInstance;
        use lycrex_tool::memory::win_memory::to_u32;
        use lycrex_tool::lycrex::logo;
        use lycrex_tool::lycrex::logger::start_log_simple;
        use lycrex_tool::info;

        logo::display_logo(Some(2));

        start_log_simple("info", true).expect("Init log failed");

        info!("example", "Init log success");

        let proc = ProcessInstance::new_by_name("tes.exe").expect("Create process instance failed");

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

    #[cfg(not(all(feature = "win-memory", target_os = "windows")))]
    {
        logo::display_logo(Some(2));
        println!("Memory feature is not enabled in this platform");
    }
}