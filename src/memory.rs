#[cfg(all(feature = "win-memory", target_os = "windows"))]
pub mod win_memory {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
    use crate::{trace, debug, info, error, warn};

    /// Read memory from a specific process
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

    pub fn print_bytes(bytes: &[u8]) {
        println!("{:02X?}", bytes);
    }

    pub fn to_u32(bytes: &[u8]) -> u32 {
        if bytes.len() >= 4 {
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        } else {
            0
        }
    }

    pub fn print_u32(bytes: &[u8]) {
        let val = to_u32(bytes);
        if val != 0 {
            println!("u32: {}", val);
        } else {
            println!("Byte length less than 4, cannot parse as u32");
        }
    }

    pub fn to_u64(bytes: &[u8]) -> u64 {
        if bytes.len() >= 8 {
            u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7]
            ])
        } else {
            0
        }
    }

    pub fn print_u64(bytes: &[u8]) {
        let val = to_u64(bytes);
        if val != 0 {
            println!("u64: {}", val);
        } else {
            println!("Byte length less than 8, cannot parse as u64");
        }
    }

    pub fn to_utf8_string(bytes: &[u8]) -> String {
        match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        }
    }

    pub fn print_utf8_string(bytes: &[u8]) {
        let s = to_utf8_string(bytes);
        if !s.is_empty() {
            println!("utf8: {}", s);
        } else {
            println!("Not a valid utf8 string");
        }
    }

    /// Find process id by process name
    pub fn find_pid_by_name(process_name: &str) -> std::io::Result<u32> {
        trace!("memory", "Find process by name: {}", process_name);
        
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS, PROCESSENTRY32, Process32First, Process32Next
        };
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map_err(|e| {
                error!("memory", "Create process snapshot failed: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create process snapshot: {}", e))
            })?;
        let mut entry = PROCESSENTRY32 { dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32, ..Default::default() };
        let mut pid = None;
        if unsafe { Process32First(snapshot, &mut entry) }.is_ok() {
            loop {
                let nul_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let exe = String::from_utf8_lossy(&entry.szExeFile[..nul_pos]);
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
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Process not found: {}", process_name)))
            }
        }
    }

    /// Read memory from the main module of a process by pid and offset
    pub fn read_process_memory_by_pid(pid: u32, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
        use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE, MODULEENTRY32, Module32First};
        let handle = unsafe { OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false, pid) }
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to open process: {}", e)))?;
        let mod_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) }
            .map_err(|e| {
                unsafe { CloseHandle(handle) }.ok();
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create module snapshot: {}", e))
            })?;
        let mut mod_entry = MODULEENTRY32 { dwSize: std::mem::size_of::<MODULEENTRY32>() as u32, ..Default::default() };
        let mut base_addr = 0usize;
        if unsafe { Module32First(mod_snapshot, &mut mod_entry) }.is_ok() {
            base_addr = mod_entry.modBaseAddr as usize;
        }
        unsafe { CloseHandle(mod_snapshot) }.ok();
        if base_addr == 0 {
            unsafe { CloseHandle(handle) }.ok();
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Module base address not found"));
        }
        let address = base_addr + offset;
        let result = read_process_memory(handle, address, size);
        unsafe { CloseHandle(handle) }.ok();
        result
    }

    /// Read memory from the main module of a process by name and offset
    pub fn read_process_memory_by_name(process_name: &str, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let pid = find_pid_by_name(process_name)?;
        read_process_memory_by_pid(pid, offset, size)
    }

    pub struct ProcessInstance {
        handle: HANDLE,
        base_addr: usize,
        pid: u32,
        name: Option<String>,
    }

    impl ProcessInstance {
        /// Create instance by process name
        pub fn new_by_name(process_name: &str) -> std::io::Result<Self> {
            info!("memory", "Create instance by name: {}", process_name);
            let pid = find_pid_by_name(process_name)?;
            let (handle, base_addr) = open_handle_and_base(pid)?;
            info!("memory", "[{}] Instance created: PID={}, BaseAddr=0x{:X}", pid, pid, base_addr);
            Ok(Self {
                handle,
                base_addr,
                pid,
                name: Some(process_name.to_string()),
            })
        }
        /// Create instance by pid
        pub fn new_by_pid(pid: u32) -> std::io::Result<Self> {
            let (handle, base_addr) = open_handle_and_base(pid)?;
            Ok(Self {
                handle,
                base_addr,
                pid,
                name: None,
            })
        }
        /// Read memory at offset
        pub fn read_memory(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
            debug!("memory", "[{}] Read memory: Offset=0x{:X}, Size={}", self.pid, offset, size);
            read_process_memory(self.handle, self.base_addr + offset, size)
        }
        /// Read u32 at offset
        pub fn read_u32(&self, offset: usize) -> std::io::Result<u32> {
            debug!("memory", "[{}] Read u32: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 4)?;
            Ok(super::win_memory::to_u32(&bytes))
        }
        /// Read u64 at offset
        pub fn read_u64(&self, offset: usize) -> std::io::Result<u64> {
            debug!("memory", "[{}] Read u64: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 8)?;
            Ok(super::win_memory::to_u64(&bytes))
        }
        /// Read utf8 string at offset
        pub fn read_utf8(&self, offset: usize, size: usize) -> std::io::Result<String> {
            debug!("memory", "[{}] Read utf8: Offset=0x{:X}, Size={}", self.pid, offset, size);
            let bytes = self.read_memory(offset, size)?;
            Ok(super::win_memory::to_utf8_string(&bytes))
        }
        pub fn pid(&self) -> u32 { self.pid }
        pub fn base_addr(&self) -> usize { self.base_addr }
        pub fn name(&self) -> Option<&str> { self.name.as_deref() }

        /// Write memory at offset
        pub fn write_memory(&self, offset: usize, data: &[u8]) -> std::io::Result<()> {
            debug!("memory", "[{}] Write memory: Offset=0x{:X}, Size={}", self.pid, offset, data.len());
            
            let mut bytes_written = 0usize;
            let result = unsafe {
                WriteProcessMemory(
                    self.handle,
                    (self.base_addr + offset) as _,
                    data.as_ptr() as _,
                    data.len(),
                    Some(&mut bytes_written as *mut usize),
                )
            };
            match result {
                Ok(_) if bytes_written == data.len() => {
                    debug!("memory", "[{}] Write memory success: Offset=0x{:X}, Written bytes={}", self.pid, offset, bytes_written);
                    Ok(())
                },
                Ok(_) => {
                    warn!("memory", "[{}] Write memory incomplete: Offset=0x{:X}, Expected={}, Actual={}", self.pid, offset, data.len(), bytes_written);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Only wrote {} bytes", bytes_written)))
                },
                Err(e) => {
                    error!("memory", "[{}] Write memory failed: Offset=0x{:X}, Error={}", self.pid, offset, e);
                    Err(std::io::Error::from_raw_os_error(e.code().0 as i32))
                },
            }
        }
        /// Write u32 at offset
        pub fn write_u32(&self, offset: usize, value: u32) -> std::io::Result<()> {
            self.write_memory(offset, &value.to_le_bytes())
        }
        /// Write u64 at offset
        pub fn write_u64(&self, offset: usize, value: u64) -> std::io::Result<()> {
            self.write_memory(offset, &value.to_le_bytes())
        }
        /// Write utf8 string at offset
        pub fn write_utf8(&self, offset: usize, s: &str) -> std::io::Result<()> {
            self.write_memory(offset, s.as_bytes())
        }
    }

    impl Drop for ProcessInstance {
        fn drop(&mut self) {
            use windows::Win32::Foundation::CloseHandle;
            debug!("memory", "[{}] Destroy process instance", self.pid);
            unsafe { CloseHandle(self.handle) }.ok();
        }
    }

    /// Internal utility: open process handle and get main module base address
    fn open_handle_and_base(pid: u32) -> std::io::Result<(HANDLE, usize)> {
        trace!("memory", "[{}] Open process handle", pid);
        
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
        use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE, MODULEENTRY32, Module32First};
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
        let mut mod_entry = MODULEENTRY32 { dwSize: std::mem::size_of::<MODULEENTRY32>() as u32, ..Default::default() };
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
} 