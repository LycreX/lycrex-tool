// 启动管理模块
pub mod startup;

// 进程管理模块
pub mod process;

// 服务管理模块
pub mod service;

// 文件系统工具模块
pub mod filesystem;

// 网络工具模块
pub mod network;

// 系统信息模块
pub mod sysinfo;

// 注册表工具模块 (Windows)
#[cfg(target_os = "windows")]
pub mod registry;

// 权限管理模块
pub mod permissions;

// 任务调度模块
pub mod scheduler;

// 环境变量管理模块
pub mod environment;

// 硬件信息模块
pub mod hardware;

// 安全工具模块
pub mod security;

// 内存管理模块
pub mod memory;

// 通用工具和类型定义
pub mod common;

/// 系统工具统一入口
pub struct SystemTools;

impl SystemTools {
    /// 获取系统信息管理器
    pub fn sysinfo() -> sysinfo::SystemInfoManager {
        let sys = sysinfo::SystemInfoManager::new();
        sys.refresh();
        sys
    }
    
    /// 获取启动管理器
    pub fn startup() -> &'static startup::StartupManager {
        &startup::StartupManager
    }
    
    /// 获取进程管理器
    pub fn process() -> &'static process::ProcessManager {
        &process::ProcessManager
    }
    
    /// 获取服务管理器
    pub fn service() -> &'static service::ServiceManager {
        &service::ServiceManager
    }
    
    /// 获取文件系统工具
    pub fn filesystem() -> &'static filesystem::FileSystemTools {
        &filesystem::FileSystemTools
    }
    
    /// 获取网络工具
    pub fn network() -> &'static network::NetworkTools {
        &network::NetworkTools
    }
    
    /// 获取权限管理器
    pub fn permissions() -> &'static permissions::PermissionManager {
        &permissions::PermissionManager
    }
    
    /// 获取任务调度器
    pub fn scheduler() -> &'static scheduler::TaskScheduler {
        &scheduler::TaskScheduler
    }
    
    /// 获取环境变量管理器
    pub fn environment() -> &'static environment::EnvironmentManager {
        &environment::EnvironmentManager
    }
    
    /// 获取硬件信息工具
    pub fn hardware() -> &'static hardware::HardwareInfo {
        &hardware::HardwareInfo
    }
    
    /// 获取安全工具
    pub fn security() -> &'static security::SecurityTools {
        &security::SecurityTools
    }
    
    /// 获取内存管理器
    pub fn memory() -> memory::MemoryManager {
        memory::MemoryManager::new()
    }
    
    /// 获取注册表工具 (Windows only)
    #[cfg(target_os = "windows")]
    pub fn registry() -> &'static registry::RegistryTools {
        &registry::RegistryTools
    }
}

/// 快速访问宏
#[macro_export]
macro_rules! systools {
    (sysinfo) => { $crate::system::SystemTools::sysinfo() };
    (startup) => { $crate::system::SystemTools::startup() };
    (process) => { $crate::system::SystemTools::process() };
    (service) => { $crate::system::SystemTools::service() };
    (filesystem) => { $crate::system::SystemTools::filesystem() };
    (network) => { $crate::system::SystemTools::network() };
    (permissions) => { $crate::system::SystemTools::permissions() };
    (scheduler) => { $crate::system::SystemTools::scheduler() };
    (environment) => { $crate::system::SystemTools::environment() };
    (hardware) => { $crate::system::SystemTools::hardware() };
    (security) => { $crate::system::SystemTools::security() };
    (memory) => { $crate::system::SystemTools::memory() };
}

/// Windows专用宏
#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! sys_win {
    (registry) => { $crate::system::SystemTools::registry() };
} 