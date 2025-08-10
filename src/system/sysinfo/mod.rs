// 系统信息模块
// 使用 sysinfo 库获取跨平台系统信息

use crate::system::common::error::{SystemResult, SystemError};
use sysinfo::{System, Pid, DiskUsage, Disks, Networks};
use std::sync::{Mutex, Arc};
use std::time::{Duration, Instant};

/// 系统信息管理器
pub struct SystemInfoManager {
    system: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    networks: Arc<Mutex<Networks>>,
    last_refresh: Arc<Mutex<Instant>>,
    refresh_interval: Duration,
}

impl SystemInfoManager {
    /// 创建新的系统信息管理器
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new_all())),
            disks: Arc::new(Mutex::new(Disks::new_with_refreshed_list())),
            networks: Arc::new(Mutex::new(Networks::new_with_refreshed_list())),
            last_refresh: Arc::new(Mutex::new(Instant::now())),
            refresh_interval: Duration::from_secs(2), // 默认2秒刷新间隔
        }
    }
    
    /// 创建带自定义刷新间隔的系统信息管理器    
    pub fn with_refresh_interval(refresh_interval: Duration) -> Self {
        let mut manager = Self::new();
        manager.refresh_interval = refresh_interval;
        manager
    }
    
    /// 检查是否需要刷新数据
    fn should_refresh(&self) -> bool {
        if let Ok(last_refresh) = self.last_refresh.lock() {
            last_refresh.elapsed() >= self.refresh_interval
        } else {
            true // 如果无法获取锁，则强制刷新
        }
    }
    
    /// 更新最后刷新时间
    fn update_refresh_time(&self) {
        if let Ok(mut last_refresh) = self.last_refresh.lock() {
            *last_refresh = Instant::now();
        }
    }
    
    /// 智能刷新系统信息（只在需要时刷新）
    pub fn smart_refresh(&self) -> SystemResult<()> {
        if self.should_refresh() {
            self.refresh();
            Ok(())
        } else {
            Ok(())
        }
    }
    
    /// 强制刷新系统信息
    pub fn refresh(&self) {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_all();
        }
        if let Ok(mut disks) = self.disks.lock() {
            disks.refresh(true);
        }
        if let Ok(mut networks) = self.networks.lock() {
            networks.refresh(true);
        }
        self.update_refresh_time();
    }
    
    /// 仅刷新网络信息（用于需要实时网络数据的场景）
    pub fn refresh_networks(&self) {
        if let Ok(mut networks) = self.networks.lock() {
            networks.refresh(true);
        }
    }
    
    /// 仅刷新CPU信息（用于需要实时CPU数据的场景）
    pub fn refresh_cpu(&self) {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_cpu_usage();
        }
    }
    
    /// 仅刷新内存信息
    pub fn refresh_memory(&self) {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_memory();
        }
    }
    
    /// 获取基本系统信息
    pub fn get_basic_info(&self) -> SystemResult<BasicSystemInfo> {
        Ok(BasicSystemInfo {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            os_family: std::env::consts::FAMILY.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            username: self.get_current_user()?,
            uptime: System::uptime(),
            boot_time: System::boot_time(),
        })
    }
    
    /// 获取当前用户
    pub fn get_current_user(&self) -> SystemResult<String> {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .or_else(|_| std::env::var("LOGNAME"))
            .map_err(|_| SystemError::NotFound("Unable to determine current user".to_string()))
    }
    
    /// 获取内存信息
    pub fn get_memory_info(&self) -> SystemResult<MemoryInfo> {
        // 智能刷新内存数据
        self.refresh_memory();
        
        let system = self.system.lock().map_err(|_| {
            SystemError::Internal("Failed to lock system info".to_string())
        })?;
        
        let total = system.total_memory();
        let used = system.used_memory();
        let available = system.available_memory();
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        
        Ok(MemoryInfo {
            total,
            used,
            available,
            usage_percent,
            swap_total: system.total_swap(),
            swap_used: system.used_swap(),
        })
    }
    
    /// 获取CPU信息
    pub fn get_cpu_info(&self) -> SystemResult<Vec<CpuInfo>> {
        // 智能刷新CPU数据
        self.refresh_cpu();
        
        let system = self.system.lock().map_err(|_| {
            SystemError::Internal("Failed to lock system info".to_string())
        })?;
        
        let cpus: Vec<CpuInfo> = system.cpus().iter().enumerate().map(|(index, cpu)| {
            CpuInfo {
                name: cpu.name().to_string(),
                brand: cpu.brand().to_string(),
                frequency: cpu.frequency(),
                usage: cpu.cpu_usage(),
                vendor_id: cpu.vendor_id().to_string(),
                core_index: index,
            }
        }).collect();
        
        Ok(cpus)
    }
    
    /// 获取磁盘信息
    pub fn get_disk_info(&self) -> SystemResult<Vec<DiskInfo>> {
        // 智能刷新磁盘数据
        self.smart_refresh()?;
        
        let disks = self.disks.lock().map_err(|_| {
            SystemError::Internal("Failed to lock disk info".to_string())
        })?;
        
        let disk_list: Vec<DiskInfo> = disks.iter().map(|disk| {
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                used_space: disk.total_space() - disk.available_space(),
                filesystem: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
                disk_type: format!("{:?}", disk.kind()),
            }
        }).collect();
        
        Ok(disk_list)
    }
    
    /// 获取网络接口信息（自动刷新以获取最新数据）
    pub fn get_network_info(&self) -> SystemResult<Vec<NetworkInterfaceInfo>> {
        // 强制刷新网络数据以获取最新统计信息
        self.refresh_networks();
        
        let networks = self.networks.lock().map_err(|_| {
            SystemError::Internal("Failed to lock network info".to_string())
        })?;
        
        let network_list: Vec<NetworkInterfaceInfo> = networks.iter().map(|(name, network)| {
            NetworkInterfaceInfo {
                name: name.clone(),
                bytes_received: network.received(),
                bytes_transmitted: network.transmitted(),
                packets_received: network.packets_received(),
                packets_transmitted: network.packets_transmitted(),
                errors_on_received: network.errors_on_received(),
                errors_on_transmitted: network.errors_on_transmitted(),
                mac_address: network.mac_address().to_string(),
            }
        }).collect();
        
        Ok(network_list)
    }
    
    /// 获取活跃的网络接口信息（过滤掉没有活动的接口）
    pub fn get_active_network_info(&self) -> SystemResult<Vec<NetworkInterfaceInfo>> {
        let all_networks = self.get_network_info()?;
        
        // 过滤出有网络活动的接口
        let active_networks: Vec<NetworkInterfaceInfo> = all_networks
            .into_iter()
            .filter(|net| {
                net.bytes_received > 0 || 
                net.bytes_transmitted > 0 || 
                net.packets_received > 0 || 
                net.packets_transmitted > 0
            })
            .collect();
        
        Ok(active_networks)
    }
    
    /// 获取主要网络接口（通常是有最多流量的接口）
    pub fn get_primary_network_interface(&self) -> SystemResult<Option<NetworkInterfaceInfo>> {
        let networks = self.get_active_network_info()?;
        
        // 找到流量最大的接口（排除回环接口）
        let primary = networks
            .into_iter()
            .filter(|net| !net.name.starts_with("lo") && !net.name.starts_with("utun"))
            .max_by_key(|net| net.bytes_received + net.bytes_transmitted);
        
        Ok(primary)
    }
    
    /// 获取进程列表
    pub fn get_processes(&self) -> SystemResult<Vec<ProcessInfo>> {
        // 智能刷新系统数据
        self.smart_refresh()?;
        
        let system = self.system.lock().map_err(|_| {
            SystemError::Internal("Failed to lock system info".to_string())
        })?;
        
        let processes: Vec<ProcessInfo> = system.processes().iter().map(|(pid, process)| {
            // 处理 OsStr 到 String 的转换
            let name = process.name().to_string_lossy().to_string();
            let command_line = process.cmd().iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            
            ProcessInfo {
                pid: pid.as_u32(),
                name,
                executable_path: process.exe().map(|p| p.to_string_lossy().to_string()),
                command_line: Some(command_line),
                parent_pid: process.parent().map(|p| p.as_u32()),
                memory_usage: Some(process.memory()),
                virtual_memory: Some(process.virtual_memory()),
                cpu_usage: Some(process.cpu_usage()),
                start_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time())),
                run_time: process.run_time(),
                user: process.user_id().map(|uid| uid.to_string()),
                status: ProcessStatus::Running, // sysinfo doesn't provide detailed status
                disk_usage: process.disk_usage(),
            }
        }).collect();
        
        Ok(processes)
    }
    
    /// 获取前N个占用内存最多的进程
    pub fn get_top_memory_processes(&self, limit: usize) -> SystemResult<Vec<ProcessInfo>> {
        let mut processes = self.get_processes()?;
        
        // 按内存使用排序
        processes.sort_by(|a, b| {
            b.memory_usage.unwrap_or(0).cmp(&a.memory_usage.unwrap_or(0))
        });
        
        processes.truncate(limit);
        Ok(processes)
    }
    
    /// 获取前N个占用CPU最多的进程
    pub fn get_top_cpu_processes(&self, limit: usize) -> SystemResult<Vec<ProcessInfo>> {
        let mut processes = self.get_processes()?;
        
        // 按CPU使用率排序
        processes.sort_by(|a, b| {
            b.cpu_usage.unwrap_or(0.0).partial_cmp(&a.cpu_usage.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        processes.truncate(limit);
        Ok(processes)
    }
    
    /// 根据PID获取进程信息
    pub fn get_process_by_pid(&self, pid: u32) -> SystemResult<Option<ProcessInfo>> {
        self.smart_refresh()?;
        
        let system = self.system.lock().map_err(|_| {
            SystemError::Internal("Failed to lock system info".to_string())
        })?;
        
        if let Some(process) = system.process(Pid::from(pid as usize)) {
            let name = process.name().to_string_lossy().to_string();
            let command_line = process.cmd().iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            
            Ok(Some(ProcessInfo {
                pid,
                name,
                executable_path: process.exe().map(|p| p.to_string_lossy().to_string()),
                command_line: Some(command_line),
                parent_pid: process.parent().map(|p| p.as_u32()),
                memory_usage: Some(process.memory()),
                virtual_memory: Some(process.virtual_memory()),
                cpu_usage: Some(process.cpu_usage()),
                start_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time())),
                run_time: process.run_time(),
                user: process.user_id().map(|uid| uid.to_string()),
                status: ProcessStatus::Running,
                disk_usage: process.disk_usage(),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 根据进程名搜索进程
    pub fn find_processes_by_name(&self, name: &str) -> SystemResult<Vec<ProcessInfo>> {
        let processes = self.get_processes()?;
        let matching_processes: Vec<ProcessInfo> = processes
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
            .collect();
        
        Ok(matching_processes)
    }
    
    /// 获取系统负载信息
    pub fn get_load_average(&self) -> SystemResult<LoadAverage> {
        let load_avg = System::load_average();
        Ok(LoadAverage {
            one_minute: load_avg.one,
            five_minutes: load_avg.five,
            fifteen_minutes: load_avg.fifteen,
        })
    }
    
    /// 获取系统性能摘要
    pub fn get_performance_summary(&self) -> SystemResult<PerformanceSummary> {
        let memory = self.get_memory_info()?;
        let cpus = self.get_cpu_info()?;
        let load = self.get_load_average()?;
        let active_networks = self.get_active_network_info()?;
        
        // 计算平均CPU使用率
        let avg_cpu_usage = if !cpus.is_empty() {
            cpus.iter().map(|cpu| cpu.usage).sum::<f32>() / cpus.len() as f32
        } else {
            0.0
        };
        
        // 计算总网络流量
        let total_network_rx = active_networks.iter().map(|n| n.bytes_received).sum();
        let total_network_tx = active_networks.iter().map(|n| n.bytes_transmitted).sum();
        
        Ok(PerformanceSummary {
            cpu_usage_percent: avg_cpu_usage,
            memory_usage_percent: memory.usage_percent,
            memory_total: memory.total,
            memory_used: memory.used,
            cpu_core_count: cpus.len(),
            load_average_1min: load.one_minute,
            active_network_interfaces: active_networks.len(),
            total_network_received: total_network_rx,
            total_network_transmitted: total_network_tx,
            uptime: System::uptime(),
        })
    }
    
    /// 获取用户列表
    pub fn get_users(&self) -> SystemResult<Vec<UserInfo>> {
        // sysinfo 0.36 中 users() 方法可能已经移除或改变
        // 暂时返回空列表，或者使用其他方式获取用户信息
        Ok(vec![])
    }
    
    /// 检查是否有管理员权限
    pub fn has_admin_privileges(&self) -> bool {
        crate::system::common::utils::SystemUtils::has_admin_privileges()
    }
    
    /// 格式化内存大小
    pub fn format_memory_size(&self, bytes: u64) -> String {
        crate::system::common::utils::SystemUtils::format_bytes(bytes)
    }
    
    /// 获取刷新间隔
    pub fn get_refresh_interval(&self) -> Duration {
        self.refresh_interval
    }
    
    /// 设置刷新间隔
    pub fn set_refresh_interval(&mut self, interval: Duration) {
        self.refresh_interval = interval;
    }
    
    /// 获取上次刷新时间距现在的时长
    pub fn time_since_last_refresh(&self) -> Duration {
        if let Ok(last_refresh) = self.last_refresh.lock() {
            last_refresh.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 基本系统信息
#[derive(Debug, Clone)]
pub struct BasicSystemInfo {
    pub os_name: String,         // 操作系统名称
    pub os_version: String,      // 操作系统版本
    pub kernel_version: String,  // 内核版本
    pub os_family: String,       // 操作系统系列
    pub arch: String,            // 架构
    pub hostname: String,        // 主机名
    pub username: String,        // 当前用户名
    pub uptime: u64,            // 系统运行时间（秒）
    pub boot_time: u64,         // 系统启动时间（Unix时间戳）
}

/// 内存信息
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,             // 总内存（字节）
    pub used: u64,              // 已使用内存（字节）
    pub available: u64,         // 可用内存（字节）
    pub usage_percent: f32,     // 使用率百分比
    pub swap_total: u64,        // 交换空间总大小（字节）
    pub swap_used: u64,         // 已使用交换空间（字节）
}

/// CPU信息
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,           // CPU名称
    pub brand: String,          // CPU品牌
    pub frequency: u64,         // 频率（MHz）
    pub usage: f32,             // 使用率百分比
    pub vendor_id: String,      // 厂商ID
    pub core_index: usize,      // 核心索引
}

/// 磁盘信息
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,           // 磁盘名称
    pub mount_point: String,    // 挂载点
    pub total_space: u64,       // 总空间（字节）
    pub available_space: u64,   // 可用空间（字节）
    pub used_space: u64,        // 已使用空间（字节）
    pub filesystem: String,     // 文件系统类型
    pub is_removable: bool,     // 是否可移动
    pub disk_type: String,      // 磁盘类型
}

/// 网络接口信息
#[derive(Debug, Clone)]
pub struct NetworkInterfaceInfo {
    pub name: String,                    // 接口名称
    pub bytes_received: u64,             // 接收字节数
    pub bytes_transmitted: u64,          // 发送字节数
    pub packets_received: u64,           // 接收数据包数
    pub packets_transmitted: u64,        // 发送数据包数
    pub errors_on_received: u64,         // 接收错误数
    pub errors_on_transmitted: u64,      // 发送错误数
    pub mac_address: String,             // MAC地址
}

/// 进程信息（扩展版本）
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub executable_path: Option<String>,
    pub command_line: Option<String>,
    pub parent_pid: Option<u32>,
    pub memory_usage: Option<u64>,       // 物理内存使用（字节）
    pub virtual_memory: Option<u64>,     // 虚拟内存使用（字节）
    pub cpu_usage: Option<f32>,          // CPU使用率百分比
    pub start_time: Option<std::time::SystemTime>,
    pub run_time: u64,                   // 运行时间（秒）
    pub user: Option<String>,
    pub status: ProcessStatus,
    pub disk_usage: DiskUsage,           // 磁盘使用情况
}

// 导入 ProcessStatus
use crate::system::common::types::ProcessStatus;

/// 系统负载平均值
#[derive(Debug, Clone)]
pub struct LoadAverage {
    pub one_minute: f64,        // 1分钟负载
    pub five_minutes: f64,      // 5分钟负载
    pub fifteen_minutes: f64,   // 15分钟负载
}

/// 用户信息
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub name: String,           // 用户名
    pub groups: Vec<String>,    // 所属组
} 

/// 系统性能摘要
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub cpu_core_count: usize,
    pub load_average_1min: f64,
    pub active_network_interfaces: usize,
    pub total_network_received: u64,
    pub total_network_transmitted: u64,
    pub uptime: u64,
} 