// 启动管理模块
// 管理系统启动项，支持Windows、Linux、macOS

use crate::system::common::*;

// 平台特定实现
// #[cfg(target_os = "windows")]
// pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

// 通用类型和接口
pub mod types;

// 重新导出
pub use types::*;

/// 启动管理器统一接口
pub struct StartupManager;

impl StartupManager {
    /// 获取所有启动项
    pub fn list_all(&self) -> SystemResult<Vec<StartupEntry>> {
        // 根据平台调用相应实现
        #[cfg(target_os = "windows")]
        // return windows::list_all_startup_entries();
        return Err(SystemError::NotSupported("Unsupported platform".to_string()));

        
        #[cfg(target_os = "linux")]
        return linux::list_all_startup_entries();
        
        #[cfg(target_os = "macos")]
        return macos::list_all_startup_entries();
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(SystemError::NotSupported("Unsupported platform".to_string()))
    }
    
    /// 添加启动项
    pub fn add_entry(&self, _entry: &StartupEntry) -> SystemResult<()> {
        #[cfg(target_os = "windows")]
        // return windows::add_startup_entry(_entry);
        return Err(SystemError::NotSupported("Unsupported platform".to_string()));
        
        #[cfg(target_os = "linux")]
        return linux::add_startup_entry(_entry);
        
        #[cfg(target_os = "macos")]
        return macos::add_startup_entry(_entry);
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(SystemError::NotSupported("Unsupported platform".to_string()))
    }
    
    /// 移除启动项
    pub fn remove_entry(&self, id: &str, _startup_type: StartupType) -> SystemResult<()> {
        // #[cfg(target_os = "windows")]
        // return windows::remove_startup_entry(id, _startup_type);
        return Err(SystemError::NotSupported("Unsupported platform".to_string()));
        
        #[cfg(target_os = "linux")]
        return linux::remove_startup_entry(id, _startup_type);
        
        #[cfg(target_os = "macos")]
        return macos::remove_startup_entry(id, _startup_type);
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(SystemError::NotSupported("Unsupported platform".to_string()))
    }
    
    /// 获取支持的启动类型
    pub fn get_supported_types(&self) -> Vec<StartupType> {
        #[cfg(target_os = "windows")]
        return vec![
            StartupType::RegistryCurrentUser,
            StartupType::RegistryLocalMachine,
            StartupType::StartupFolder,
            StartupType::WindowsService,
            StartupType::TaskScheduler,
        ];
        
        #[cfg(target_os = "linux")]
        return vec![
            StartupType::SystemdUser,
            StartupType::SystemdSystem,
            StartupType::DesktopAutostart,
        ];
        
        #[cfg(target_os = "macos")]
        return vec![
            StartupType::LaunchAgent,
            StartupType::LaunchDaemon,
            StartupType::LoginItems,
        ];
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        vec![]
    }
} 