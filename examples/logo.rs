#[cfg(all(feature = "win-memory", target_os = "windows"))]

pub fn main() {
    use lycrex_tool::memory::win_memory::ProcessInstance;
    use lycrex_tool::memory::win_memory::to_u32;
    use lycrex_tool::lycrex::logo;

    logo::print_logo(Some(10));

    loop {
        use std::time::Duration;
        use std::thread;

        thread::sleep(Duration::from_secs(1));

        let proc = ProcessInstance::new_by_name("tes.exe").expect("无法创建进程实例");
        proc.write_u32(0x23000, 666).expect("写入内存失败");
        let bytes = proc.read_memory(0x23000, 16).expect("读取内存失败");
        let val = to_u32(&bytes);
        println!("val: {}", val);
    }
}