use std::fmt;

/// 系统工具错误类型
#[derive(Debug, Clone)]
pub enum SystemError {
    /// 权限不足
    PermissionDenied(String),
    /// 资源不存在
    NotFound(String),
    /// 操作不支持
    NotSupported(String),
    /// 系统调用失败
    SystemCall(String, Option<i32>),
    /// IO错误
    Io(String),
    /// 解析错误
    Parse(String),
    /// 网络错误
    Network(String),
    /// 配置错误
    Configuration(String),
    /// 超时错误
    Timeout(String),
    /// 资源忙碌
    Busy(String),
    /// 无效参数
    InvalidArgument(String),
    /// 内部错误
    Internal(String),
    /// 进程相关错误
    ProcessError(String),
    /// 内存操作错误
    MemoryError(String),
    /// 未知错误
    Unknown(String),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            SystemError::NotFound(msg) => write!(f, "Not found: {}", msg),
            SystemError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            SystemError::SystemCall(msg, code) => {
                if let Some(code) = code {
                    write!(f, "System call failed: {} (code: {})", msg, code)
                } else {
                    write!(f, "System call failed: {}", msg)
                }
            }
            SystemError::Io(msg) => write!(f, "IO error: {}", msg),
            SystemError::Parse(msg) => write!(f, "Parse error: {}", msg),
            SystemError::Network(msg) => write!(f, "Network error: {}", msg),
            SystemError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            SystemError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            SystemError::Busy(msg) => write!(f, "Resource busy: {}", msg),
            SystemError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            SystemError::Internal(msg) => write!(f, "Internal error: {}", msg),
            SystemError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            SystemError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            SystemError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for SystemError {}

impl From<std::io::Error> for SystemError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                SystemError::PermissionDenied(err.to_string())
            }
            std::io::ErrorKind::NotFound => SystemError::NotFound(err.to_string()),
            std::io::ErrorKind::TimedOut => SystemError::Timeout(err.to_string()),
            std::io::ErrorKind::InvalidInput => SystemError::InvalidArgument(err.to_string()),
            _ => SystemError::Io(err.to_string()),
        }
    }
}

pub type SystemResult<T> = Result<T, SystemError>;

#[macro_export]
macro_rules! system_error {
    (permission_denied, $msg:expr) => {
        $crate::system::common::SystemError::PermissionDenied($msg.to_string())
    };
    (not_found, $msg:expr) => {
        $crate::system::common::SystemError::NotFound($msg.to_string())
    };
    (not_supported, $msg:expr) => {
        $crate::system::common::SystemError::NotSupported($msg.to_string())
    };
    (system_call, $msg:expr) => {
        $crate::system::common::SystemError::SystemCall($msg.to_string(), None)
    };
    (system_call, $msg:expr, $code:expr) => {
        $crate::system::common::SystemError::SystemCall($msg.to_string(), Some($code))
    };
    (io, $msg:expr) => {
        $crate::system::common::SystemError::Io($msg.to_string())
    };
    (parse, $msg:expr) => {
        $crate::system::common::SystemError::Parse($msg.to_string())
    };
    (network, $msg:expr) => {
        $crate::system::common::SystemError::Network($msg.to_string())
    };
    (config, $msg:expr) => {
        $crate::system::common::SystemError::Configuration($msg.to_string())
    };
    (timeout, $msg:expr) => {
        $crate::system::common::SystemError::Timeout($msg.to_string())
    };
    (busy, $msg:expr) => {
        $crate::system::common::SystemError::Busy($msg.to_string())
    };
    (invalid_arg, $msg:expr) => {
        $crate::system::common::SystemError::InvalidArgument($msg.to_string())
    };
    (internal, $msg:expr) => {
        $crate::system::common::SystemError::Internal($msg.to_string())
    };
    (process, $msg:expr) => {
        $crate::system::common::SystemError::ProcessError($msg.to_string())
    };
    (memory, $msg:expr) => {
        $crate::system::common::SystemError::MemoryError($msg.to_string())
    };
    (unknown, $msg:expr) => {
        $crate::system::common::SystemError::Unknown($msg.to_string())
    };
} 