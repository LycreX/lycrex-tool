use crate::system::common::error::{SystemResult, SystemError};
use crate::system::common::types::{OperatingSystem, PermissionStatus};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// 系统工具通用函数
pub struct SystemUtils;

impl SystemUtils {
    /// 检查是否有管理员/root权限
    pub fn has_admin_privileges() -> bool {
        match OperatingSystem::current() {
            OperatingSystem::Windows => {
                // Windows: 检查是否是管理员
                #[cfg(target_os = "windows")]
                {
                    use windows::Win32::Foundation::BOOL;
                    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION};
                    use windows::Win32::System::Threading::GetCurrentProcess;
                    use windows::Win32::Foundation::CloseHandle;
                    
                    unsafe {
                        let mut token = std::mem::zeroed();
                        if windows::Win32::Security::OpenProcessToken(
                            GetCurrentProcess(),
                            windows::Win32::Security::TOKEN_QUERY,
                            &mut token,
                        ).is_ok() {
                            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: BOOL(0) };
                            let mut size = 0;
                            
                            if GetTokenInformation(
                                token,
                                TokenElevation,
                                Some(&mut elevation as *mut _ as *mut _),
                                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                                &mut size,
                            ).is_ok() {
                                CloseHandle(token).ok();
                                return elevation.TokenIsElevated.as_bool();
                            }
                            CloseHandle(token).ok();
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            }
            OperatingSystem::Linux | OperatingSystem::MacOS => {
                // Unix-like: 检查是否是root用户
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    unsafe { libc::getuid() == 0 }
                }
                #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// 获取当前用户名
    pub fn get_current_user() -> SystemResult<String> {
        match std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .or_else(|_| std::env::var("LOGNAME"))
        {
            Ok(user) => Ok(user),
            Err(_) => {
                // 尝试其他方法获取用户名
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    use std::ffi::CStr;
                    unsafe {
                        let uid = libc::getuid();
                        let passwd = libc::getpwuid(uid);
                        if !passwd.is_null() {
                            let name = CStr::from_ptr((*passwd).pw_name);
                            if let Ok(name_str) = name.to_str() {
                                return Ok(name_str.to_string());
                            }
                        }
                    }
                }
                Err(SystemError::NotFound("Unable to determine current user".to_string()))
            }
        }
    }
    
    /// 执行系统命令
    pub fn execute_command(command: &str, args: &[&str]) -> SystemResult<String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| SystemError::SystemCall(format!("Failed to execute command: {} - {}", command, e), None))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(SystemError::SystemCall(
                format!("Command failed: {}", stderr),
                output.status.code(),
            ))
        }
    }
    
    /// 检查命令是否存在
    pub fn command_exists(command: &str) -> bool {
        match OperatingSystem::current() {
            OperatingSystem::Windows => {
                Command::new("where")
                    .arg(command)
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            _ => {
                Command::new("which")
                    .arg(command)
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
        }
    }
    
    /// 格式化字节大小
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
    
    /// 格式化时间戳
    pub fn format_timestamp(timestamp: SystemTime) -> String {
        match timestamp.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                if let Some(dt) = chrono::DateTime::from_timestamp(secs as i64, 0) {
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                } else {
                    "Unknown".to_string()
                }
            }
            Err(_) => "Unknown".to_string(),
        }
    }
    
    /// 解析命令行
    pub fn parse_command_line(command_line: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = command_line.split_whitespace().collect();
        if parts.is_empty() {
            return (String::new(), Vec::new());
        }
        
        let command = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        (command, args)
    }
    
    /// 生成唯一ID
    pub fn generate_id(prefix: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{}_{}", prefix, timestamp)
    }
    
    /// 验证路径是否存在
    pub fn path_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    
    /// 验证路径是否是可执行文件
    pub fn is_executable(path: &str) -> bool {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return false;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = path.metadata() {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0;
            }
        }
        
        #[cfg(windows)]
        {
            // Windows下检查文件扩展名
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                return matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "scr");
            }
        }
        
        false
    }
    
    /// 获取系统临时目录
    pub fn get_temp_dir() -> String {
        std::env::temp_dir().to_string_lossy().to_string()
    }
    
    /// 检查权限状态
    pub fn check_permission_status(requires_admin: bool) -> PermissionStatus {
        if !requires_admin {
            return PermissionStatus::HasPermission;
        }
        
        if Self::has_admin_privileges() {
            PermissionStatus::HasPermission
        } else {
            PermissionStatus::RequiresElevation
        }
    }
}

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 截断字符串到指定长度
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
    
    /// 移除字符串两端的空白字符
    pub fn trim_whitespace(s: &str) -> String {
        s.trim().to_string()
    }
    
    /// 将字符串转换为安全的文件名
    pub fn to_safe_filename(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }
} 