// 内存管理模块
// 提供跨平台的内存读写、进程内存操作等功能

use crate::system::common::*;

// 平台特定的内存操作模块
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

// 通用内存工具
pub mod utils;

/// 内存管理器
pub struct MemoryManager;

impl MemoryManager {
    /// 创建新的内存管理器实例
    pub fn new() -> Self {
        Self
    }

    /// 根据进程名创建进程内存实例
    pub fn create_process_instance_by_name(&self, process_name: &str) -> SystemResult<ProcessMemoryInstance> {
        ProcessMemoryInstance::new_by_name(process_name)
    }

    /// 根据PID创建进程内存实例
    pub fn create_process_instance_by_pid(&self, pid: u32) -> SystemResult<ProcessMemoryInstance> {
        ProcessMemoryInstance::new_by_pid(pid)
    }

    /// 获取进程内存信息
    pub fn get_process_memory_info(&self, pid: u32) -> SystemResult<ProcessMemoryInfo> {
        #[cfg(target_os = "windows")]
        return windows::get_process_memory_info(pid);
        
        #[cfg(target_os = "macos")]
        return macos::get_process_memory_info(pid);
        
        #[cfg(target_os = "linux")]
        return linux::get_process_memory_info(pid);
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 根据进程名查找PID
    pub fn find_pid_by_name(&self, process_name: &str) -> SystemResult<u32> {
        #[cfg(target_os = "windows")]
        return windows::find_pid_by_name(process_name);
        
        #[cfg(target_os = "macos")]
        return macos::find_pid_by_name(process_name);
        
        #[cfg(target_os = "linux")]
        return linux::find_pid_by_name(process_name);
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 列出所有进程的内存信息
    pub fn list_process_memory_info(&self) -> SystemResult<Vec<ProcessMemoryInfo>> {
        #[cfg(target_os = "windows")]
        return windows::list_process_memory_info();
        
        #[cfg(target_os = "macos")]
        return macos::list_process_memory_info();
        
        #[cfg(target_os = "linux")]
        return linux::list_process_memory_info();
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }
}

/// 进程内存操作实例
/// 封装对特定进程的内存读写操作
pub struct ProcessMemoryInstance {
    pid: u32,
    process_name: Option<String>,
    base_address: usize,
    #[cfg(target_os = "windows")]
    handle: windows::ProcessHandle,
    #[cfg(target_os = "macos")]
    task: macos::TaskPort,
}

impl ProcessMemoryInstance {
    /// 通过进程名创建实例
    pub fn new_by_name(process_name: &str) -> SystemResult<Self> {
        #[cfg(target_os = "windows")]
        return windows::ProcessInstance::new_by_name(process_name)
            .map_err(|e| SystemError::ProcessError(format!("Failed to create Windows process instance: {}", e)))
            .and_then(|instance| Ok(Self::from_windows_instance(instance)));
        
        #[cfg(target_os = "macos")]
        return macos::ProcessInstance::new_by_name(process_name)
            .map_err(|e| SystemError::ProcessError(format!("Failed to create macOS process instance: {}", e)))
            .and_then(|instance| Ok(Self::from_macos_instance(instance)));
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 通过PID创建实例
    pub fn new_by_pid(pid: u32) -> SystemResult<Self> {
        #[cfg(target_os = "windows")]
        return windows::ProcessInstance::new_by_pid(pid)
            .map_err(|e| SystemError::ProcessError(format!("Failed to create Windows process instance: {}", e)))
            .and_then(|instance| Ok(Self::from_windows_instance(instance)));
        
        #[cfg(target_os = "macos")]
        return macos::ProcessInstance::new_by_pid(pid)
            .map_err(|e| SystemError::ProcessError(format!("Failed to create macOS process instance: {}", e)))
            .and_then(|instance| Ok(Self::from_macos_instance(instance)));
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 读取内存
    pub fn read_memory(&self, offset: usize, size: usize) -> SystemResult<Vec<u8>> {
        #[cfg(target_os = "windows")]
        return windows::read_process_memory(self.handle, self.base_address + offset, size)
            .map_err(|e| SystemError::MemoryError(format!("Windows memory read failed: {}", e)));
        
        #[cfg(target_os = "macos")]
        return macos::read_process_memory(self.task, self.base_address + offset, size)
            .map_err(|e| SystemError::MemoryError(format!("macOS memory read failed: {}", e)));
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 写入内存
    pub fn write_memory(&self, offset: usize, data: &[u8]) -> SystemResult<MemoryOperationResult> {
        #[cfg(target_os = "windows")]
        return windows::write_process_memory(self.handle, self.base_address + offset, data)
            .map(|bytes_written| MemoryOperationResult {
                success: bytes_written == data.len(),
                bytes_processed: bytes_written,
                error_message: if bytes_written != data.len() { 
                    Some(format!("Only wrote {} of {} bytes", bytes_written, data.len()))
                } else { None },
            })
            .map_err(|e| SystemError::MemoryError(format!("Windows memory write failed: {}", e)));
        
        #[cfg(target_os = "macos")]
        return macos::write_process_memory(self.task, self.base_address + offset, data)
            .map(|bytes_written| MemoryOperationResult {
                success: bytes_written == data.len(),
                bytes_processed: bytes_written,
                error_message: if bytes_written != data.len() { 
                    Some(format!("Only wrote {} of {} bytes", bytes_written, data.len()))
                } else { None },
            })
            .map_err(|e| SystemError::MemoryError(format!("macOS memory write failed: {}", e)));
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        Err(SystemError::NotSupported("Platform not supported".to_string()))
    }

    /// 读取 u32 值
    pub fn read_u32(&self, offset: usize) -> SystemResult<u32> {
        let bytes = self.read_memory(offset, 4)?;
        Ok(utils::bytes_to_u32(&bytes))
    }

    /// 读取 u64 值
    pub fn read_u64(&self, offset: usize) -> SystemResult<u64> {
        let bytes = self.read_memory(offset, 8)?;
        Ok(utils::bytes_to_u64(&bytes))
    }

    /// 读取 UTF-8 字符串
    pub fn read_utf8_string(&self, offset: usize, size: usize) -> SystemResult<String> {
        let bytes = self.read_memory(offset, size)?;
        Ok(utils::bytes_to_utf8_string(&bytes))
    }

    /// 写入 u32 值
    pub fn write_u32(&self, offset: usize, value: u32) -> SystemResult<MemoryOperationResult> {
        self.write_memory(offset, &value.to_le_bytes())
    }

    /// 写入 u64 值
    pub fn write_u64(&self, offset: usize, value: u64) -> SystemResult<MemoryOperationResult> {
        self.write_memory(offset, &value.to_le_bytes())
    }

    /// 写入 UTF-8 字符串
    pub fn write_utf8_string(&self, offset: usize, s: &str) -> SystemResult<MemoryOperationResult> {
        self.write_memory(offset, s.as_bytes())
    }

    /// 获取进程信息
    pub fn pid(&self) -> u32 { self.pid }
    pub fn base_address(&self) -> usize { self.base_address }
    pub fn process_name(&self) -> Option<&str> { self.process_name.as_deref() }

    // 平台特定的转换函数
    #[cfg(target_os = "windows")]
    fn from_windows_instance(instance: windows::ProcessInstance) -> Self {
        Self {
            pid: instance.pid(),
            process_name: instance.name().map(|s| s.to_string()),
            base_address: instance.base_addr(),
            handle: instance.handle(),
        }
    }

    #[cfg(target_os = "macos")]
    fn from_macos_instance(instance: macos::ProcessInstance) -> Self {
        Self {
            pid: instance.pid(),
            process_name: instance.name().map(|s| s.to_string()),
            base_address: instance.base_addr(),
            task: instance.task(),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
