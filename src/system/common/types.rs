use std::collections::HashMap;
use std::time::SystemTime;

/// 操作系统类型
#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
    Unknown(String),
}

impl OperatingSystem {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => OperatingSystem::Windows,
            "linux" => OperatingSystem::Linux,
            "macos" => OperatingSystem::MacOS,
            other => OperatingSystem::Unknown(other.to_string()),
        }
    }
}

/// 权限状态
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionStatus {
    /// 有足够权限
    HasPermission,
    /// 需要提升权限
    RequiresElevation,
    /// 当前平台不支持
    NotSupported,
    /// 权限被拒绝
    Denied,
}

/// 系统服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    Starting,
    Stopping,
    Unknown,
}

/// 系统服务启动类型
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStartType {
    Automatic,      // 自动启动
    Manual,         // 手动启动
    Disabled,       // 禁用
    AutomaticDelayed, // 延迟自动启动
}

/// 进程信息
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub executable_path: Option<String>,
    pub command_line: Option<String>,
    pub parent_pid: Option<u32>,
    pub memory_usage: Option<u64>,    // 字节
    pub cpu_usage: Option<f32>,       // 百分比
    pub start_time: Option<SystemTime>,
    pub user: Option<String>,
    pub status: ProcessStatus,
}

/// 进程状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

/// 文件权限
#[derive(Debug, Clone)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub owner: String,
    pub group: Option<String>,
    pub mode: Option<u32>,  // Unix权限模式
}

/// 网络连接信息
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub protocol: NetworkProtocol,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: Option<String>,
    pub remote_port: Option<u16>,
    pub state: ConnectionState,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

/// 网络协议
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkProtocol {
    TCP,
    UDP,
    ICMP,
    Unknown(String),
}

/// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Listen,
    Established,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    TimeWait,
    Closed,
    CloseWait,
    LastAck,
    Closing,
    Unknown,
}

/// 系统性能信息
#[derive(Debug, Clone)]
pub struct SystemPerformance {
    pub cpu_usage: f32,           // CPU使用率百分比
    pub memory_total: u64,        // 总内存 (字节)
    pub memory_used: u64,         // 已使用内存 (字节)
    pub memory_available: u64,    // 可用内存 (字节)
    pub swap_total: u64,          // 交换空间总大小 (字节)
    pub swap_used: u64,           // 已使用交换空间 (字节)
    pub disk_usage: Vec<DiskUsage>, // 磁盘使用情况
    pub network_interfaces: Vec<NetworkInterface>, // 网络接口
    pub uptime: u64,              // 系统运行时间 (秒)
    pub load_average: Option<[f32; 3]>, // 负载平均值 (1, 5, 15分钟)
}

/// 磁盘使用情况
#[derive(Debug, Clone)]
pub struct DiskUsage {
    pub device: String,           // 设备名
    pub mount_point: String,      // 挂载点
    pub total_space: u64,         // 总空间 (字节)
    pub used_space: u64,          // 已使用空间 (字节)
    pub available_space: u64,     // 可用空间 (字节)
    pub filesystem: String,       // 文件系统类型
}

/// 网络接口信息
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,             // 接口名称
    pub display_name: Option<String>, // 显示名称
    pub mac_address: Option<String>,  // MAC地址
    pub ip_addresses: Vec<String>,    // IP地址列表
    pub is_up: bool,              // 是否启用
    pub is_loopback: bool,        // 是否是回环接口
    pub bytes_sent: Option<u64>,  // 发送字节数
    pub bytes_received: Option<u64>, // 接收字节数
    pub speed: Option<u64>,       // 速度 (bps)
}

/// 环境变量
pub type EnvironmentVariables = HashMap<String, String>;

/// 任务调度信息
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub arguments: Vec<String>,
    pub schedule: TaskSchedule,
    pub enabled: bool,
    pub last_run: Option<SystemTime>,
    pub next_run: Option<SystemTime>,
    pub run_as_user: Option<String>,
}

/// 任务调度规则
#[derive(Debug, Clone)]
pub enum TaskSchedule {
    Once(SystemTime),             // 一次性任务
    Daily(u8, u8),                // 每日 (小时, 分钟)
    Weekly(u8, u8, u8),           // 每周 (星期几, 小时, 分钟)
    Monthly(u8, u8, u8),          // 每月 (日期, 小时, 分钟)
    Interval(u64),                // 间隔执行 (秒)
    Cron(String),                 // Cron表达式
    OnBoot,                       // 开机启动
    OnLogin,                      // 登录时启动
}

/// 内存读写权限
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPermission {
    Read,
    Write,
    Execute,
    ReadWrite,
    ReadExecute,
    WriteExecute,
    ReadWriteExecute,
}

/// 内存操作结果
#[derive(Debug, Clone)]
pub struct MemoryOperationResult {
    pub success: bool,
    pub bytes_processed: usize,
    pub error_message: Option<String>,
}

/// 进程内存信息
#[derive(Debug, Clone)]
pub struct ProcessMemoryInfo {
    pub pid: u32,
    pub process_name: Option<String>,
    pub base_address: usize,
    pub memory_usage: u64,      // 字节
    pub virtual_size: u64,      // 虚拟内存大小
    pub working_set: u64,       // 工作集大小
    pub peak_working_set: u64,  // 峰值工作集
    pub private_bytes: u64,     // 私有字节数
}

/// 内存区域信息
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base_address: usize,
    pub size: usize,
    pub protection: MemoryPermission,
    pub is_committed: bool,
    pub is_private: bool,
    pub module_name: Option<String>,
} 