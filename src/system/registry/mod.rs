// Windows注册表工具模块
#[cfg(target_os = "windows")]
use crate::system::common::*;

#[cfg(target_os = "windows")]
pub struct RegistryTools;

#[cfg(target_os = "windows")]
impl RegistryTools {
    pub fn read_value(&self, _hkey: &str, _subkey: &str, _value_name: &str) -> SystemResult<String> {
        Err(SystemError::NotSupported("Not implemented yet".to_string()))
    }
} 