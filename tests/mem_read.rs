#[cfg(all(feature = "win-memory", target_os = "windows"))]
#[test]
fn test_read_tes_exe_23000() {
    use lycrex_tool::win_memory::ProcessInstance;
    use lycrex_tool::win_memory::to_u32;
    let proc = ProcessInstance::new_by_name("tes.exe").expect("无法创建进程实例");
    proc.write_u32(0x23000, 666).expect("写入内存失败");
    let bytes = proc.read_memory(0x23000, 16).expect("读取内存失败");
    let val = to_u32(&bytes);
    assert!(!bytes.is_empty(), "读取到的字节为空");
    // 资源自动管理：proc 离开作用域时自动关闭句柄，无需手动释放
}