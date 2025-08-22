#[cfg(target_os = "windows")]
pub mod implementation {
    use windows::Win32::Foundation::{HANDLE, CloseHandle};
    use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS, PROCESSENTRY32, Process32First, Process32Next,
        TH32CS_SNAPMODULE, MODULEENTRY32, Module32First
    };
    use crate::{trace, debug, info, error, warn};
    use crate::system::common::*;

    pub type ProcessHandle = HANDLE;

    /// 读取进程内存
    pub fn read_process_memory(
        process_handle: HANDLE,
        address: usize,
        size: usize,
    ) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0usize;
        let success = unsafe {
            ReadProcessMemory(
                process_handle,
                address as _,
                buffer.as_mut_ptr() as _,
                size,
                Some(&mut bytes_read as *mut usize),
            )
        };
        match success {
            Ok(_) => {
                buffer.truncate(bytes_read);
                Ok(buffer)
            },
            Err(e) => Err(std::io::Error::from_raw_os_error(e.code().0 as i32)),
        }
    }

    /// 写入进程内存
    pub fn write_process_memory(
        process_handle: HANDLE,
        address: usize,
        data: &[u8],
    ) -> std::io::Result<usize> {
        let mut bytes_written = 0usize;
        let result = unsafe {
            WriteProcessMemory(
                process_handle,
                address as _,
                data.as_ptr() as _,
                data.len(),
                Some(&mut bytes_written as *mut usize),
            )
        };
        match result {
            Ok(_) => Ok(bytes_written),
            Err(e) => Err(std::io::Error::from_raw_os_error(e.code().0 as i32)),
        }
    }

    /// 根据进程名查找PID
    pub fn find_pid_by_name(process_name: &str) -> SystemResult<u32> {
        trace!("memory", "Find process by name: {}", process_name);
        
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map_err(|e| {
                error!("memory", "Create process snapshot failed: {}", e);
                SystemError::ProcessError(format!("Failed to create process snapshot: {}", e))
            })?;
        
        let mut entry = PROCESSENTRY32 { 
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32, 
            ..Default::default() 
        };
        let mut pid = None;
        
        if unsafe { Process32First(snapshot, &mut entry) }.is_ok() {
            loop {
                let nul_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let exe = String::from_utf8_lossy(unsafe { 
                    std::slice::from_raw_parts(entry.szExeFile.as_ptr() as *const u8, nul_pos) 
                });
                if exe == process_name {
                    pid = Some(entry.th32ProcessID);
                    trace!("memory", "[{}] Find process: {}", entry.th32ProcessID, process_name);
                    break;
                }
                if unsafe { Process32Next(snapshot, &mut entry) }.is_err() {
                    break;
                }
            }
        }
        
        unsafe { CloseHandle(snapshot) }.ok();
        
        match pid {
            Some(pid) => Ok(pid),
            None => {
                error!("memory", "Process not found: {}", process_name);
                Err(SystemError::ProcessError(format!("Process not found: {}", process_name)))
            }
        }
    }

    /// 获取进程内存信息
    pub fn get_process_memory_info(pid: u32) -> SystemResult<ProcessMemoryInfo> {
        use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
        
        let handle = unsafe { 
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) 
        }.map_err(|e| SystemError::ProcessError(format!("Failed to open process {}: {}", pid, e)))?;
        
        let mut mem_counters = PROCESS_MEMORY_COUNTERS::default();
        let result = unsafe {
            GetProcessMemoryInfo(
                handle, 
                &mut mem_counters, 
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32
            )
        };
        
        unsafe { CloseHandle(handle) }.ok();
        
        if result.is_ok() {
            Ok(ProcessMemoryInfo {
                pid,
                process_name: None, // 可以后续通过其他方式获取
                base_address: 0, // 需要通过模块快照获取
                memory_usage: mem_counters.WorkingSetSize as u64,
                virtual_size: mem_counters.PagefileUsage as u64,
                working_set: mem_counters.WorkingSetSize as u64,
                peak_working_set: mem_counters.PeakWorkingSetSize as u64,
                private_bytes: mem_counters.PagefileUsage as u64,
            })
        } else {
            Err(SystemError::ProcessError(format!("Failed to get memory info for process {}", pid)))
        }
    }

    /// 列出所有进程的内存信息
    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map_err(|e| SystemError::ProcessError(format!("Failed to create process snapshot: {}", e)))?;
        
        let mut entry = PROCESSENTRY32 { 
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32, 
            ..Default::default() 
        };
        let mut processes = Vec::new();
        
        if unsafe { Process32First(snapshot, &mut entry) }.is_ok() {
            loop {
                if let Ok(mem_info) = get_process_memory_info(entry.th32ProcessID) {
                    let nul_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                    let process_name = String::from_utf8_lossy(unsafe { 
                        std::slice::from_raw_parts(entry.szExeFile.as_ptr() as *const u8, nul_pos) 
                    }).to_string();
                    
                    let mut info = mem_info;
                    info.process_name = Some(process_name);
                    processes.push(info);
                }
                
                if unsafe { Process32Next(snapshot, &mut entry) }.is_err() {
                    break;
                }
            }
        }
        
        unsafe { CloseHandle(snapshot) }.ok();
        Ok(processes)
    }

    /// 进程内存操作实例
    pub struct ProcessInstance {
        handle: HANDLE,
        base_addr: usize,
        pid: u32,
        name: Option<String>,
    }

    impl ProcessInstance {
        /// 根据进程名创建实例
        pub fn new_by_name(process_name: &str) -> std::io::Result<Self> {
            info!("memory", "Create instance by name: {}", process_name);
            let pid = find_pid_by_name(process_name)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
            let (handle, base_addr) = open_handle_and_base(pid)?;
            info!("memory", "[{}] Instance created: PID={}, BaseAddr=0x{:X}", pid, pid, base_addr);
            Ok(Self {
                handle,
                base_addr,
                pid,
                name: Some(process_name.to_string()),
            })
        }

        /// 根据PID创建实例
        pub fn new_by_pid(pid: u32) -> std::io::Result<Self> {
            let (handle, base_addr) = open_handle_and_base(pid)?;
            Ok(Self {
                handle,
                base_addr,
                pid,
                name: None,
            })
        }

        /// 读取内存
        pub fn read_memory(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
            debug!("memory", "[{}] Read memory: Offset=0x{:X}, Size={}", self.pid, offset, size);
            read_process_memory(self.handle, self.base_addr + offset, size)
        }

        /// 写入内存
        pub fn write_memory(&self, offset: usize, data: &[u8]) -> std::io::Result<()> {
            debug!("memory", "[{}] Write memory: Offset=0x{:X}, Size={}", self.pid, offset, data.len());
            
            let bytes_written = write_process_memory(self.handle, self.base_addr + offset, data)?;
            
            if bytes_written == data.len() {
                debug!("memory", "[{}] Write memory success: Offset=0x{:X}, Written bytes={}", self.pid, offset, bytes_written);
                Ok(())
            } else {
                warn!("memory", "[{}] Write memory incomplete: Offset=0x{:X}, Expected={}, Actual={}", self.pid, offset, data.len(), bytes_written);
                Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Only wrote {} bytes", bytes_written)))
            }
        }

        /// 读取 u32
        pub fn read_u32(&self, offset: usize) -> std::io::Result<u32> {
            debug!("memory", "[{}] Read u32: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 4)?;
            Ok(crate::system::memory::utils::bytes_to_u32(&bytes))
        }

        /// 读取 u64
        pub fn read_u64(&self, offset: usize) -> std::io::Result<u64> {
            debug!("memory", "[{}] Read u64: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 8)?;
            Ok(crate::system::memory::utils::bytes_to_u64(&bytes))
        }

        /// 读取 UTF-8 字符串
        pub fn read_utf8(&self, offset: usize, size: usize) -> std::io::Result<String> {
            debug!("memory", "[{}] Read utf8: Offset=0x{:X}, Size={}", self.pid, offset, size);
            let bytes = self.read_memory(offset, size)?;
            Ok(crate::system::memory::utils::bytes_to_utf8_string(&bytes))
        }

        /// 写入 u32
        pub fn write_u32(&self, offset: usize, value: u32) -> std::io::Result<()> {
            self.write_memory(offset, &value.to_le_bytes())
        }

        /// 写入 u64
        pub fn write_u64(&self, offset: usize, value: u64) -> std::io::Result<()> {
            self.write_memory(offset, &value.to_le_bytes())
        }

        /// 写入 UTF-8 字符串
        pub fn write_utf8(&self, offset: usize, s: &str) -> std::io::Result<()> {
            self.write_memory(offset, s.as_bytes())
        }

        // Getters
        pub fn pid(&self) -> u32 { self.pid }
        pub fn base_addr(&self) -> usize { self.base_addr }
        pub fn name(&self) -> Option<&str> { self.name.as_deref() }
        pub fn handle(&self) -> HANDLE { self.handle }
    }

    impl Drop for ProcessInstance {
        fn drop(&mut self) {
            debug!("memory", "[{}] Destroy process instance", self.pid);
            unsafe { CloseHandle(self.handle) }.ok();
        }
    }

    /// 打开进程句柄并获取主模块基址
    fn open_handle_and_base(pid: u32) -> std::io::Result<(HANDLE, usize)> {
        trace!("memory", "[{}] Open process handle", pid);
        
        let handle = unsafe { OpenProcess(
            PROCESS_VM_READ | PROCESS_QUERY_INFORMATION | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            false, pid) }
            .map_err(|e| {
                error!("memory", "[{}] Open process failed: Error={}", pid, e);
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to open process: {}", e))
            })?;
        
        let mod_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) }
            .map_err(|e| {
                unsafe { CloseHandle(handle) }.ok();
                error!("memory", "[{}] Create module snapshot failed: Error={}", pid, e);
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create module snapshot: {}", e))
            })?;
        
        let mut mod_entry = MODULEENTRY32 { 
            dwSize: std::mem::size_of::<MODULEENTRY32>() as u32, 
            ..Default::default() 
        };
        let mut base_addr = 0usize;
        
        if unsafe { Module32First(mod_snapshot, &mut mod_entry) }.is_ok() {
            base_addr = mod_entry.modBaseAddr as usize;
            debug!("memory", "[{}] Get module base address: BaseAddr=0x{:X}", pid, base_addr);
        }
        
        unsafe { CloseHandle(mod_snapshot) }.ok();
        
        if base_addr == 0 {
            unsafe { CloseHandle(handle) }.ok();
            error!("memory", "[{}] Module base address not found", pid);
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Module base address not found"));
        }
        
        Ok((handle, base_addr))
    }

    /// 根据进程名和偏移读取内存
    pub fn read_process_memory_by_name(process_name: &str, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let pid = find_pid_by_name(process_name)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
        read_process_memory_by_pid(pid, offset, size)
    }

    /// 根据PID和偏移读取内存 
    pub fn read_process_memory_by_pid(pid: u32, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let (handle, base_addr) = open_handle_and_base(pid)?;
        let result = read_process_memory(handle, base_addr + offset, size);
        unsafe { CloseHandle(handle) }.ok();
        result
    }
}

#[cfg(not(target_os = "windows"))]
pub mod implementation {
    use crate::system::common::*;

    pub type ProcessHandle = usize; // 占位类型

    pub fn find_pid_by_name(_process_name: &str) -> SystemResult<u32> {
        Err(SystemError::NotSupported("Windows memory operations not enabled. Enable 'win-memory' feature.".to_string()))
    }

    pub fn get_process_memory_info(_pid: u32) -> SystemResult<ProcessMemoryInfo> {
        Err(SystemError::NotSupported("Windows memory operations not enabled. Enable 'win-memory' feature.".to_string()))
    }

    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        Err(SystemError::NotSupported("Windows memory operations not enabled. Enable 'win-memory' feature.".to_string()))
    }

    pub fn read_process_memory(_handle: ProcessHandle, _address: usize, _size: usize) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Windows memory operations not enabled"))
    }

    pub fn write_process_memory(_handle: ProcessHandle, _address: usize, _data: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Windows memory operations not enabled"))
    }

    pub struct ProcessInstance;

    impl ProcessInstance {
        pub fn new_by_name(_process_name: &str) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Windows memory operations not enabled"))
        }

        pub fn new_by_pid(_pid: u32) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Windows memory operations not enabled"))
        }

        pub fn pid(&self) -> u32 { 0 }
        pub fn base_addr(&self) -> usize { 0 }
        pub fn name(&self) -> Option<&str> { None }
        pub fn handle(&self) -> ProcessHandle { 0 }
    }
}

// 重新导出实现
pub use implementation::*;
