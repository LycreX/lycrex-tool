#[cfg(target_os = "linux")]
pub mod implementation {
    use crate::system::common::*;
    
    pub type ProcessHandle = i32;

    /// 根据进程名查找 PID
    pub fn find_pid_by_name(process_name: &str) -> SystemResult<u32> {
        use std::process::Command;
        
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(process_name)
            .output()
            .map_err(|e| SystemError::ProcessError(format!("Failed to execute pgrep: {}", e)))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<&str> = output_str.trim().split('\n').collect();
            
            if let Some(pid_str) = pids.first() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    return Ok(pid);
                }
            }
        }

        Err(SystemError::ProcessError(format!("Process not found: {}", process_name)))
    }

    /// 获取进程内存信息
    pub fn get_process_memory_info(pid: u32) -> SystemResult<ProcessMemoryInfo> {
        Err(SystemError::NotSupported("Linux memory operations not implemented yet".to_string()))
    }

    /// 列出所有进程的内存信息
    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        Err(SystemError::NotSupported("Linux memory operations not implemented yet".to_string()))
    }

    /// 读取进程内存
    pub fn read_process_memory(
        _handle: ProcessHandle,
        _address: usize,
        _size: usize,
    ) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations not implemented yet"))
    }

    /// 写入进程内存
    pub fn write_process_memory(
        _handle: ProcessHandle,
        _address: usize,
        _data: &[u8],
    ) -> std::io::Result<usize> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations not implemented yet"))
    }

    pub struct ProcessInstance;

    impl ProcessInstance {
        pub fn new_by_name(_process_name: &str) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations not implemented yet"))
        }

        pub fn new_by_pid(_pid: u32) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations not implemented yet"))
        }

        pub fn pid(&self) -> u32 { 0 }
        pub fn base_addr(&self) -> usize { 0 }
        pub fn name(&self) -> Option<&str> { None }
        pub fn handle(&self) -> ProcessHandle { -1 }
    }
}

#[cfg(not(target_os = "linux"))]
pub mod implementation {
    use crate::system::common::*;

    pub type ProcessHandle = i32;

    pub fn find_pid_by_name(_process_name: &str) -> SystemResult<u32> {
        Err(SystemError::NotSupported("Linux memory operations only available on Linux".to_string()))
    }

    pub fn get_process_memory_info(_pid: u32) -> SystemResult<ProcessMemoryInfo> {
        Err(SystemError::NotSupported("Linux memory operations only available on Linux".to_string()))
    }

    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        Err(SystemError::NotSupported("Linux memory operations only available on Linux".to_string()))
    }

    pub fn read_process_memory(_handle: ProcessHandle, _address: usize, _size: usize) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations only available on Linux"))
    }

    pub fn write_process_memory(_handle: ProcessHandle, _address: usize, _data: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations only available on Linux"))
    }

    pub struct ProcessInstance;

    impl ProcessInstance {
        pub fn new_by_name(_process_name: &str) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations only available on Linux"))
        }

        pub fn new_by_pid(_pid: u32) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Linux memory operations only available on Linux"))
        }

        pub fn pid(&self) -> u32 { 0 }
        pub fn base_addr(&self) -> usize { 0 }
        pub fn name(&self) -> Option<&str> { None }
        pub fn handle(&self) -> ProcessHandle { -1 }
    }
}

pub use implementation::*;
