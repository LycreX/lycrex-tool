// 硬件信息模块
use crate::system::common::*;

pub struct HardwareInfo;

impl HardwareInfo {
    pub fn get_cpu_info(&self) -> SystemResult<CpuInfo> {
        Err(SystemError::NotSupported("Not implemented yet".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub frequency: u64,
} 