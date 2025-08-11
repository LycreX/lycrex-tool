// 目前几乎完全不可用
// 仅支持读取/写入内存
// DATA/TEXT段基址获取失败

#[cfg(target_os = "macos")]
// #[deprecated(since = "0.0.0", note = "macOS memory operations are not fully implemented")]
pub mod implementation {
    use crate::system::common::*;
    use crate::{trace, debug, info, error, warn};

    type KernReturn = i32;
    type TaskT = u32;
    type VmAddress = usize;
    type VmSize = usize;
    #[allow(unused)]
    type VmOffset = u64;
    type VmProt = i32;
    type PidT = i32;
    type VmInherit = u32;
    type VmBehavior = i32;

    pub type TaskPort = TaskT;

    const KERN_SUCCESS: KernReturn = 0;
    const VM_PROT_READ: VmProt = 0x01;
    const VM_PROT_WRITE: VmProt = 0x02;
    const VM_PROT_EXECUTE: VmProt = 0x04;
    
    const VM_INHERIT_SHARE: VmInherit = 0;
    #[allow(unused)]
    const VM_INHERIT_COPY: VmInherit = 1;
    #[allow(unused)]
    const VM_INHERIT_NONE: VmInherit = 2;
    
    const MH_MAGIC_64: u32 = 0xfeedfacf;
    const MH_EXECUTE: u32 = 0x2;
    const LC_SEGMENT_64: u32 = 0x19;
    
    #[repr(C)]
    #[derive(Debug)]
    struct MachHeader64 {
        magic: u32,
        cputype: i32,
        cpusubtype: i32,
        filetype: u32,
        ncmds: u32,
        sizeofcmds: u32,
        flags: u32,
        reserved: u32,
    }
    
    #[repr(C)]
    #[derive(Debug)]
    struct LoadCommand {
        cmd: u32,
        cmdsize: u32,
    }
    
    #[repr(C)]
    #[derive(Debug)]
    struct SegmentCommand64 {
        cmd: u32,
        cmdsize: u32,
        segname: [u8; 16],
        vmaddr: u64,
        vmsize: u64,
        fileoff: u64,
        filesize: u64,
        maxprot: i32,
        initprot: i32,
        nsects: u32,
        flags: u32,
    }

    #[repr(C)]
    #[derive(Debug)]
    struct VmRegionBasicInfo64 {
        protection: VmProt,
        max_protection: VmProt,
        inheritance: VmInherit,
        shared: u32,
        reserved: u32,
        offset: u64,
        behavior: VmBehavior,
        user_wired_count: u16,
    }

    extern "C" {
        fn task_for_pid(target_task: TaskT, pid: PidT, task: *mut TaskT) -> KernReturn;
        fn mach_task_self() -> TaskT;
        fn vm_read_overwrite(
            target_task: TaskT,
            address: VmAddress,
            size: VmSize,
            data: VmAddress,
            data_count: *mut VmSize,
        ) -> KernReturn;
        fn vm_write(
            target_task: TaskT,
            address: VmAddress,
            data: VmAddress,
            data_count: VmSize,
        ) -> KernReturn;
        fn vm_protect(
            target_task: TaskT,
            address: VmAddress,
            size: VmSize,
            set_maximum: bool,
            new_protection: VmProt,
        ) -> KernReturn;
        fn vm_region_64(
            target_task: TaskT,
            address: *mut VmAddress,
            size: *mut VmSize,
            flavor: u32,
            info: *mut VmRegionBasicInfo64,
            info_count: *mut u32,
            object_name: *mut u32,
        ) -> KernReturn;
        #[allow(unused)]
        fn getpid() -> PidT;
    }

    /// 根据进程名查找 PID
    pub fn find_pid_by_name(process_name: &str) -> SystemResult<u32> {
        trace!("memory", "Find process by name: {}", process_name);

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
                    trace!("memory", "[{}] Found process: {}", pid, process_name);
                    return Ok(pid);
                }
            }
        }

        error!("memory", "Process not found: {}", process_name);
        Err(SystemError::ProcessError(format!("Process not found: {}", process_name)))
    }

    /// 获取进程内存信息
    pub fn get_process_memory_info(pid: u32) -> SystemResult<ProcessMemoryInfo> {
        use std::process::Command;

        let output = Command::new("ps")
            .args(&["-o", "pid,comm,rss,vsz", "-p", &pid.to_string()])
            .output()
            .map_err(|e| SystemError::ProcessError(format!("Failed to get process info: {}", e)))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.trim().split('\n').collect();
            
            if lines.len() >= 2 {
                let data_line = lines[1];
                let fields: Vec<&str> = data_line.split_whitespace().collect();
                
                if fields.len() >= 4 {
                    let rss = fields[2].parse::<u64>().unwrap_or(0) * 1024;
                    let vsz = fields[3].parse::<u64>().unwrap_or(0) * 1024;
                    let process_name = fields[1].to_string();
                    
                    return Ok(ProcessMemoryInfo {
                        pid,
                        process_name: Some(process_name),
                        base_address: 0,
                        memory_usage: rss,
                        virtual_size: vsz,
                        working_set: rss,
                        peak_working_set: rss,
                        private_bytes: rss,
                    });
                }
            }
        }

        Err(SystemError::ProcessError(format!("Failed to get memory info for process {}", pid)))
    }

    /// 列出所有进程的内存信息
    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        use std::process::Command;

        let output = Command::new("ps")
            .args(&["-eo", "pid,comm,rss,vsz"])
            .output()
            .map_err(|e| SystemError::ProcessError(format!("Failed to list processes: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::ProcessError("Failed to execute ps command".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.trim().split('\n').collect();
        let mut processes = Vec::new();

        for line in lines.iter().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                if let Ok(pid) = fields[0].parse::<u32>() {
                    let rss = fields[2].parse::<u64>().unwrap_or(0) * 1024;
                    let vsz = fields[3].parse::<u64>().unwrap_or(0) * 1024;
                    let process_name = fields[1].to_string();
                    
                    processes.push(ProcessMemoryInfo {
                        pid,
                        process_name: Some(process_name),
                        base_address: 0,
                        memory_usage: rss,
                        virtual_size: vsz,
                        working_set: rss,
                        peak_working_set: rss,
                        private_bytes: rss,
                    });
                }
            }
        }

        Ok(processes)
    }

    /// 读取进程内存
    pub fn read_process_memory(
        task: TaskPort,
        address: usize,
        size: usize,
    ) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut data_count = size;
        
        let kern_return = unsafe {
            vm_read_overwrite(
                task,
                address,
                size,
                buffer.as_mut_ptr() as VmAddress,
                &mut data_count,
            )
        };

        if kern_return == KERN_SUCCESS {
            buffer.truncate(data_count);
            Ok(buffer)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("vm_read_overwrite failed with code: {}", kern_return)
            ))
        }
    }

    /// 写入进程内存
    pub fn write_process_memory(
        task: TaskPort,
        address: usize,
        data: &[u8],
    ) -> std::io::Result<usize> {
        let kern_return = unsafe {
            vm_protect(
                task,
                address,
                data.len(),
                false,
                VM_PROT_READ | VM_PROT_WRITE,
            )
        };

        if kern_return != KERN_SUCCESS {
            debug!("memory", "vm_protect failed, continuing anyway: {}", kern_return);
        }

        let kern_return = unsafe {
            vm_write(
                task,
                address,
                data.as_ptr() as VmAddress,
                data.len(),
            )
        };

        if kern_return == KERN_SUCCESS {
            Ok(data.len())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("vm_write failed with code: {}", kern_return)
            ))
        }
    }

    /// 获取进程的 task port
    fn get_task_for_pid(pid: u32) -> std::io::Result<TaskPort> {
        let mut task: TaskPort = 0;
        let kern_return = unsafe {
            task_for_pid(mach_task_self(), pid as PidT, &mut task)
        };

        if kern_return == KERN_SUCCESS {
            Ok(task)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("task_for_pid failed with code: {}. Note: This usually requires root privileges or proper entitlements.", kern_return)
            ))
        }
    }

    /// 获取进程主模块基址
    /// 通过遍历内存区域找到可执行的 Mach-O 头部
    fn get_main_module_base_address(pid: u32) -> std::io::Result<usize> {
        trace!("memory", "[{}] Getting main module base address", pid);
        
        let task = get_task_for_pid(pid)?;
        
        let search_addresses = [
            0x100000000usize,
            0x10000usize,
            0x1000usize,
        ];
        
        for &start_addr in &search_addresses {
            if let Ok(base_addr) = find_main_module_from_address(task, start_addr) {
                info!("memory", "[{}] Found main module base address: 0x{:X}", pid, base_addr);
                return Ok(base_addr);
            }
        }
        
        match find_main_module_by_vm_region(task, pid) {
            Ok(base_addr) => {
                info!("memory", "[{}] Found main module via vm_region: 0x{:X}", pid, base_addr);
                Ok(base_addr)
            }
            Err(e) => {
                error!("memory", "[{}] Failed to find main module base address: {}", pid, e);
                warn!("memory", "[{}] Using fallback address 0x100000000", pid);
                Ok(0x100000000)
            }
        }
    }
    
    /// 从指定地址开始查找主模块，返回 TEXT 段基址（实际加载基址）
    fn find_main_module_from_address(task: TaskPort, start_addr: usize) -> std::io::Result<usize> {
        match read_process_memory(task, start_addr, std::mem::size_of::<MachHeader64>()) {
            Ok(header_data) => {
                if header_data.len() >= std::mem::size_of::<MachHeader64>() {
                    let magic = u32::from_le_bytes([
                        header_data[0], header_data[1], 
                        header_data[2], header_data[3]
                    ]);
                    
                    let filetype = u32::from_le_bytes([
                        header_data[12], header_data[13], 
                        header_data[14], header_data[15]
                    ]);
                    
                    if magic == MH_MAGIC_64 && filetype == MH_EXECUTE {
                        debug!("memory", "Found valid Mach-O header at 0x{:X}", start_addr);
                        
                        let ncmds = u32::from_le_bytes([
                            header_data[16], header_data[17], 
                            header_data[18], header_data[19]
                        ]);
                        let sizeofcmds = u32::from_le_bytes([
                            header_data[20], header_data[21], 
                            header_data[22], header_data[23]
                        ]);
                        
                        if let Ok(text_base) = calculate_text_base_address(task, start_addr, ncmds, sizeofcmds) {
                            info!("memory", "Calculated TEXT segment base address: 0x{:X}", text_base);
                            return Ok(text_base);
                        }
                        
                        info!("memory", "Using Mach-O header address as base: 0x{:X}", start_addr);
                        return Ok(start_addr);
                    }
                }
            }
            Err(_) => {}
        }
        
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No valid Mach-O header found at 0x{:X}", start_addr)
        ))
    }
    
    /// 计算TEXT段的实际基址
    /// 通过比较Mach-O中TEXT段的虚拟地址和实际加载地址来计算偏移
    fn calculate_text_base_address(task: TaskPort, mach_header_addr: usize, ncmds: u32, sizeofcmds: u32) -> std::io::Result<usize> {
        let header_size = std::mem::size_of::<MachHeader64>();
        let load_commands_start = mach_header_addr + header_size;
        
        // 读取所有 load commands
        match read_process_memory(task, load_commands_start, sizeofcmds as usize) {
            Ok(cmd_data) => {
                let mut offset = 0usize;
                
                for _ in 0..ncmds {
                    if offset + std::mem::size_of::<LoadCommand>() > cmd_data.len() {
                        break;
                    }
                    
                    // 读取 load command 头部
                    let cmd = u32::from_le_bytes([
                        cmd_data[offset], cmd_data[offset + 1],
                        cmd_data[offset + 2], cmd_data[offset + 3]
                    ]);
                    let cmdsize = u32::from_le_bytes([
                        cmd_data[offset + 4], cmd_data[offset + 5],
                        cmd_data[offset + 6], cmd_data[offset + 7]
                    ]);
                    
                    if cmd == LC_SEGMENT_64 && offset + std::mem::size_of::<SegmentCommand64>() <= cmd_data.len() {
                        // 读取段名
                        let segname_start = offset + 8;
                        let segname_end = segname_start + 16;
                        if segname_end <= cmd_data.len() {
                            let segname = &cmd_data[segname_start..segname_end];
                            
                            // 检查是否是 __TEXT 段
                            if segname.starts_with(b"__TEXT\0") {
                                // 读取虚拟地址
                                let vmaddr_start = offset + 24;
                                if vmaddr_start + 8 <= cmd_data.len() {
                                    let vmaddr = u64::from_le_bytes([
                                        cmd_data[vmaddr_start], cmd_data[vmaddr_start + 1],
                                        cmd_data[vmaddr_start + 2], cmd_data[vmaddr_start + 3],
                                        cmd_data[vmaddr_start + 4], cmd_data[vmaddr_start + 5],
                                        cmd_data[vmaddr_start + 6], cmd_data[vmaddr_start + 7],
                                    ]);
                                    
                                    // 计算ASLR滑动偏移：实际加载地址 - 原始虚拟地址
                                    let slide = mach_header_addr.wrapping_sub(vmaddr as usize);
                                    // TEXT段的实际基址 = 原始虚拟基址 + ASLR滑动偏移
                                    let actual_text_base = vmaddr as usize + slide;
                                    debug!("memory", "Found __TEXT segment: original_vmaddr=0x{:X}, loaded_at=0x{:X}, slide=0x{:X}, actual_base=0x{:X}", 
                                        vmaddr, mach_header_addr, slide, actual_text_base);
                                    return Ok(actual_text_base);
                                }
                            }
                        }
                    }
                    
                    offset += cmdsize as usize;
                }
            }
            Err(e) => {
                debug!("memory", "Failed to read load commands for base calculation: {}", e);
            }
        }
        
        // 如果找不到TEXT段，返回Mach-O头部地址
        Ok(mach_header_addr)
    }

    /// 查找 DATA 段基址
    #[allow(unused)]
    fn find_data_segment(task: TaskPort, text_base: usize, ncmds: u32, sizeofcmds: u32) -> std::io::Result<usize> {
        let header_size = std::mem::size_of::<MachHeader64>();
        let load_commands_start = text_base + header_size;
        
        // 读取所有 load commands
        match read_process_memory(task, load_commands_start, sizeofcmds as usize) {
            Ok(cmd_data) => {
                let mut offset = 0usize;
                
                for _ in 0..ncmds {
                    if offset + std::mem::size_of::<LoadCommand>() > cmd_data.len() {
                        break;
                    }
                    
                    // 读取 load command 头部
                    let cmd = u32::from_le_bytes([
                        cmd_data[offset], cmd_data[offset + 1],
                        cmd_data[offset + 2], cmd_data[offset + 3]
                    ]);
                    let cmdsize = u32::from_le_bytes([
                        cmd_data[offset + 4], cmd_data[offset + 5],
                        cmd_data[offset + 6], cmd_data[offset + 7]
                    ]);
                    
                    if cmd == LC_SEGMENT_64 && offset + std::mem::size_of::<SegmentCommand64>() <= cmd_data.len() {
                        // 读取段名
                        let segname_start = offset + 8;
                        let segname_end = segname_start + 16;
                        if segname_end <= cmd_data.len() {
                            let segname = &cmd_data[segname_start..segname_end];
                            
                            // 检查是否是 __DATA 段
                            if segname.starts_with(b"__DATA\0") {
                                // 读取虚拟地址
                                let vmaddr_start = offset + 24;
                                if vmaddr_start + 8 <= cmd_data.len() {
                                    let vmaddr = u64::from_le_bytes([
                                        cmd_data[vmaddr_start], cmd_data[vmaddr_start + 1],
                                        cmd_data[vmaddr_start + 2], cmd_data[vmaddr_start + 3],
                                        cmd_data[vmaddr_start + 4], cmd_data[vmaddr_start + 5],
                                        cmd_data[vmaddr_start + 6], cmd_data[vmaddr_start + 7],
                                    ]);
                                    
                                    info!("memory", "Found __DATA segment at 0x{:X}", vmaddr);
                                    return Ok(vmaddr as usize);
                                }
                            }
                        }
                    }
                    
                    offset += cmdsize as usize;
                }
            }
            Err(e) => {
                debug!("memory", "Failed to read load commands: {}", e);
            }
        }
        
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DATA segment not found"
        ))
    }
    
    /// 通过 vm_region 遍历内存区域查找主模块
    fn find_main_module_by_vm_region(task: TaskPort, pid: u32) -> std::io::Result<usize> {
        const VM_REGION_BASIC_INFO_64: u32 = 9;
        
        let mut address: VmAddress = 0x1000; // 从较低地址开始
        let mut checked_count = 0;
        const MAX_REGIONS_TO_CHECK: usize = 100; // 限制检查的区域数量
        
        debug!("memory", "[{}] Starting vm_region scan for main module", pid);
        
        while checked_count < MAX_REGIONS_TO_CHECK {
            let mut size: VmSize = 0;
            let mut info = VmRegionBasicInfo64 {
                protection: 0,
                max_protection: 0,
                inheritance: VM_INHERIT_SHARE,
                shared: 0,
                reserved: 0,
                offset: 0,
                behavior: 0,
                user_wired_count: 0,
            };
            let mut info_count = std::mem::size_of::<VmRegionBasicInfo64>() as u32 / 4;
            let mut object_name: u32 = 0;
            
            let kern_return = unsafe {
                vm_region_64(
                    task,
                    &mut address,
                    &mut size,
                    VM_REGION_BASIC_INFO_64,
                    &mut info,
                    &mut info_count,
                    &mut object_name,
                )
            };
            
            if kern_return != KERN_SUCCESS {
                break;
            }
            
            // 检查是否是可执行区域（TEXT 段）
            if (info.protection & VM_PROT_EXECUTE) != 0 && size >= 0x1000 {
                debug!("memory", "[{}] Checking executable region at 0x{:X}, size: 0x{:X}", pid, address, size);
                
                // 尝试在这个区域查找 Mach-O 头部并解析 DATA 段
                if let Ok(data_base_addr) = find_main_module_from_address(task, address) {
                    return Ok(data_base_addr);
                }
            }
            
            // 移动到下一个区域
            address += size;
            checked_count += 1;
            
            // 避免地址溢出
            if address == 0 {
                break;
            }
        }
        
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Main module not found after checking {} regions", checked_count)
        ))
    }

    /// 进程内存操作实例
    pub struct ProcessInstance {
        task: TaskPort,
        base_addr: usize,
        pid: u32,
        name: Option<String>,
    }

    impl ProcessInstance {
        /// 根据进程名创建实例
        pub fn new_by_name(process_name: &str) -> std::io::Result<Self> {
            info!("memory", "Create macOS instance by name: {}", process_name);
            
            let pid = find_pid_by_name(process_name)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
            
            let task = get_task_for_pid(pid)?;
            let base_addr = get_main_module_base_address(pid)?;
            
            info!("memory", "[{}] macOS instance created: PID={}, BaseAddr=0x{:X}", pid, pid, base_addr);
            
            Ok(Self {
                task,
                base_addr,
                pid,
                name: Some(process_name.to_string()),
            })
        }

        /// 根据PID创建实例
        pub fn new_by_pid(pid: u32) -> std::io::Result<Self> {
            info!("memory", "Create macOS instance by PID: {}", pid);
            
            let task = get_task_for_pid(pid)?;
            let base_addr = get_main_module_base_address(pid)?;
            
            Ok(Self {
                task,
                base_addr,
                pid,
                name: None,
            })
        }

        /// 读取内存（相对于模块基址的偏移）
        pub fn read_memory(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
            debug!("memory", "[{}] Read memory: Offset=0x{:X}, Size={}", self.pid, offset, size);
            read_process_memory(self.task, self.base_addr + offset, size)
        }

        /// 读取绝对地址的内存
        pub fn read_memory_at(&self, address: usize, size: usize) -> std::io::Result<Vec<u8>> {
            debug!("memory", "[{}] Read memory at absolute address: Addr=0x{:X}, Size={}", self.pid, address, size);
            read_process_memory(self.task, address, size)
        }

        /// 写入内存
        pub fn write_memory(&self, offset: usize, data: &[u8]) -> std::io::Result<()> {
            debug!("memory", "[{}] Write memory: Offset=0x{:X}, Size={}", self.pid, offset, data.len());
            
            let bytes_written = write_process_memory(self.task, self.base_addr + offset, data)?;
            
            if bytes_written == data.len() {
                debug!("memory", "[{}] Write memory success: Offset=0x{:X}, Written bytes={}", self.pid, offset, bytes_written);
                Ok(())
            } else {
                warn!("memory", "[{}] Write memory incomplete: Offset=0x{:X}, Expected={}, Actual={}", self.pid, offset, data.len(), bytes_written);
                Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Only wrote {} bytes", bytes_written)))
            }
        }

        /// 读取 u32（相对偏移）
        pub fn read_u32(&self, offset: usize) -> std::io::Result<u32> {
            debug!("memory", "[{}] Read u32: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 4)?;
            Ok(crate::system::memory::utils::bytes_to_u32(&bytes))
        }

        /// 读取 u64（相对偏移）
        pub fn read_u64(&self, offset: usize) -> std::io::Result<u64> {
            debug!("memory", "[{}] Read u64: Offset=0x{:X}", self.pid, offset);
            let bytes = self.read_memory(offset, 8)?;
            Ok(crate::system::memory::utils::bytes_to_u64(&bytes))
        }

        /// 读取绝对地址的 u32
        pub fn read_u32_at(&self, address: usize) -> std::io::Result<u32> {
            debug!("memory", "[{}] Read u32 at absolute address: Addr=0x{:X}", self.pid, address);
            let bytes = self.read_memory_at(address, 4)?;
            Ok(crate::system::memory::utils::bytes_to_u32(&bytes))
        }

        /// 读取绝对地址的 u64
        pub fn read_u64_at(&self, address: usize) -> std::io::Result<u64> {
            debug!("memory", "[{}] Read u64 at absolute address: Addr=0x{:X}", self.pid, address);
            let bytes = self.read_memory_at(address, 8)?;
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
        pub fn task(&self) -> TaskPort { self.task }
    }

    /// 根据进程名和偏移读取内存
    pub fn read_process_memory_by_name(process_name: &str, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let pid = find_pid_by_name(process_name)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
        read_process_memory_by_pid(pid, offset, size)
    }

    /// 根据PID和偏移读取内存
    pub fn read_process_memory_by_pid(pid: u32, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let task = get_task_for_pid(pid)?;
        let base_addr = get_main_module_base_address(pid)?;
        read_process_memory(task, base_addr + offset, size)
    }
}

// 非 macOS 平台的空实现
#[cfg(not(target_os = "macos"))]
pub mod implementation {
    use crate::system::common::*;

    pub type TaskPort = usize;

    pub fn find_pid_by_name(_process_name: &str) -> SystemResult<u32> {
        Err(SystemError::NotSupported("macOS memory operations only available on macOS".to_string()))
    }

    pub fn get_process_memory_info(_pid: u32) -> SystemResult<ProcessMemoryInfo> {
        Err(SystemError::NotSupported("macOS memory operations only available on macOS".to_string()))
    }

    pub fn list_process_memory_info() -> SystemResult<Vec<ProcessMemoryInfo>> {
        Err(SystemError::NotSupported("macOS memory operations only available on macOS".to_string()))
    }

    pub fn read_process_memory(_task: TaskPort, _address: usize, _size: usize) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "macOS memory operations only available on macOS"))
    }

    pub fn write_process_memory(_task: TaskPort, _address: usize, _data: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "macOS memory operations only available on macOS"))
    }

    pub struct ProcessInstance;

    impl ProcessInstance {
        pub fn new_by_name(_process_name: &str) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "macOS memory operations only available on macOS"))
        }

        pub fn new_by_pid(_pid: u32) -> std::io::Result<Self> {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "macOS memory operations only available on macOS"))
        }

        pub fn pid(&self) -> u32 { 0 }
        pub fn base_addr(&self) -> usize { 0 }
        pub fn name(&self) -> Option<&str> { None }
        pub fn task(&self) -> TaskPort { 0 }
    }
}

// 重新导出实现
pub use implementation::*;
