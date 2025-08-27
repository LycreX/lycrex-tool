use crate::system::common::*;
use crate::system::startup::types::*;

/// 获取所有启动项
pub fn list_all_startup_entries() -> SystemResult<Vec<StartupEntry>> {
    Ok(vec![])
}

/// 添加启动项
pub fn add_startup_entry(_entry: &StartupEntry) -> SystemResult<()> {
    Err(SystemError::NotSupported("Linux startup management not yet implemented".to_string()))
}

/// 移除启动项
pub fn remove_startup_entry(_id: &str, _startup_type: StartupType) -> SystemResult<()> {
    Err(SystemError::NotSupported("Linux startup management not yet implemented".to_string()))
} 