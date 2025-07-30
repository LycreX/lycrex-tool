use std::{
    fmt::{self, Write},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use crate::utils::time::{TimeFormat, TimeUtils};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Level::Trace => "\x1b[90m", // 灰
            Level::Debug => "\x1b[36m", // 青
            Level::Info => "\x1b[32m",  // 绿
            Level::Warn => "\x1b[33m",  // 黄
            Level::Error => "\x1b[31m", // 红
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 日志记录结构
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub timestamp: u64,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub module_path: Option<String>,
}

/// 日志格式化器
pub trait Formatter: Send + Sync {
    fn format(&self, record: &LogRecord) -> String;
}

pub struct DefaultFormatter {
    pub use_colors: bool,
    pub show_timestamp: bool,
    pub show_target: bool,
    pub show_location: bool,
    pub time_format: TimeFormat,
    pub uptime_level: i8,
}

impl DefaultFormatter {
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_timestamp: true,
            show_target: true,
            show_location: true,
            time_format: TimeFormat::SystemTime,
            uptime_level: 3,
        }
    }

    pub fn without_colors() -> Self {
        Self {
            use_colors: false,
            show_timestamp: true,
            show_target: true,
            show_location: true,
            time_format: TimeFormat::SystemTime,
            uptime_level: 1,
        }
    }

    pub fn with_time_format(time_format: TimeFormat) -> Self {
        Self {
            use_colors: true,
            show_timestamp: true,
            show_target: true,
            show_location: true,
            time_format,
            uptime_level: 1,
        }
    }

    pub fn with_time_format_and_options(
        time_format: TimeFormat,
        use_colors: bool,
        show_timestamp: bool,
        show_target: bool,
        show_location: bool,
    ) -> Self {
        Self {
            use_colors,
            show_timestamp,
            show_target,
            show_location,
            time_format,
            uptime_level: 1,
        }
    }
}

impl Formatter for DefaultFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut output = String::new();
        
        // 时间戳
        if self.show_timestamp {
            let time_str = match self.time_format {
                TimeFormat::Unix => record.timestamp.to_string(),
                TimeFormat::UnixMillis => (record.timestamp * 1000).to_string(),
                TimeFormat::SystemTime => TimeUtils::system_time_string(),
                TimeFormat::LocalTime => TimeUtils::local_time_string(),
                TimeFormat::Iso8601 => TimeUtils::iso8601_time_string(),
                TimeFormat::Relative => {
                    if self.uptime_level < 0 {
                        TimeUtils::program_uptime_string()
                    } else {
                        let uptime = TimeUtils::program_uptime(self.uptime_level as u8);

                        let unit = match self.uptime_level {
                            0 => "s",
                            1 => "ms",
                            2 => "μs",
                            3 => "ns",
                            _ => "ms",
                        };

                        format!("+{} {}", uptime, unit)
                    }
                }
            };
            write!(&mut output, "[{}] ", time_str).unwrap();
        }

        // 级别
        if self.use_colors {
            write!(&mut output, "{}{:<5}{}\x1b[0m ", 
                record.level.color_code(), 
                record.level.as_str(),
                "\x1b[0m"
            ).unwrap();
        } else {
            write!(&mut output, "[{:<5}] ", record.level.as_str()).unwrap();
        }

        // 目标
        if self.show_target && !record.target.is_empty() {
            write!(&mut output, "({}) ", record.target).unwrap();
        }

        // 位置
        if self.show_location {
            if let (Some(file), Some(line)) = (&record.file, record.line) {
                write!(&mut output, "{}:{} ", file, line).unwrap();
            }
        }

        output.push_str(&record.message);
        output
    }
}

/// 日志输出器
pub trait Writer: Send + Sync {
    fn write(&self, record: &LogRecord);
}

/// 控制台输出器
pub struct ConsoleWriter {
    formatter: Box<dyn Formatter>,
}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {
            formatter: Box::new(DefaultFormatter::new()),
        }
    }

    pub fn with_formatter(formatter: Box<dyn Formatter>) -> Self {
        Self { formatter }
    }
}

impl Writer for ConsoleWriter {
    fn write(&self, record: &LogRecord) {
        let message = self.formatter.format(record);
        println!("{}", message);
    }
}

/// 文件输出器
pub struct FileWriter {
    path: String,
    formatter: Box<dyn Formatter>,
}

impl FileWriter {
    pub fn new(path: String) -> std::io::Result<Self> {
        Ok(Self {
            path,
            formatter: Box::new(DefaultFormatter::without_colors()),
        })
    }

    pub fn with_formatter(path: String, formatter: Box<dyn Formatter>) -> std::io::Result<Self> {
        Ok(Self { path, formatter })
    }
}

impl Writer for FileWriter {
    fn write(&self, record: &LogRecord) {
        let message = self.formatter.format(record);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            use std::io::Write;
            let _ = writeln!(file, "{}", message);
        }
    }
}

/// 日志配置
pub struct LogConfig {
    pub level: Level,
    pub writers: Vec<Box<dyn Writer>>,
    pub time_format: TimeFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::Info,
            writers: vec![],
            time_format: TimeFormat::SystemTime,
        }
    }
}

/// 日志记录器
pub struct Logger {
    config: LogConfig,
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
        }
    }

    /// 从配置创建日志记录器
    pub fn from_config(config: LogConfig) -> Self {
        Self { config }
    }

    /// 设置日志级别
    pub fn set_level(&mut self, level: Level) {
        self.config.level = level;
    }

    /// 添加输出器
    pub fn add_writer(&mut self, writer: Box<dyn Writer>) {
        self.config.writers.push(writer);
    }

    /// 清空所有输出器
    pub fn clear_writers(&mut self) {
        self.config.writers.clear();
    }

    /// 记录日志核心函数
    pub fn log(&self, level: Level, target: &str, message: &str, 
                file: Option<&str>, line: Option<u32>, module_path: Option<&str>) {
        if level < self.config.level {
            return;
        }

        let record = LogRecord {
            level,
            target: target.to_string(),
            message: message.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            file: file.map(|s| s.to_string()),
            line,
            module_path: module_path.map(|s| s.to_string()),
        };
        
        for writer in &self.config.writers {
            writer.write(&record);
        }
    }
}

/// 全局日志记录器
static LOGGER: Mutex<Option<Arc<Logger>>> = Mutex::new(None);

/// 初始化全局日志记录器
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(Arc::new(logger));
    Ok(())
}

/// 初始化带配置的全局日志记录器
pub fn init_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::from_config(config);
    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(Arc::new(logger));
    Ok(())
}

/// 获取全局日志记录器
fn get_logger() -> Option<Arc<Logger>> {
    LOGGER.lock().unwrap().clone()
}

/// 记录日志的内部函数
pub fn log(level: Level, target: &str, message: &str, 
            file: Option<&str>, line: Option<u32>, module_path: Option<&str>) {
    if let Some(logger) = get_logger() {
        logger.log(level, target, message, file, line, module_path);
    }
}

/// 日志宏
#[macro_export]
macro_rules! log {
    ($level:expr, $target:expr, $($arg:tt)*) => {
        $crate::lycrex::logger::log(
            $level,
            $target,
            &format!($($arg)*),
            Some(file!()),
            Some(line!()),
            Some(module_path!())
        );
    };
}

#[macro_export]
macro_rules! trace {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::Trace, $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::Debug, $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::Info, $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::Warn, $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::Error, $target, $($arg)*);
    };
}

/// 默认日志宏
#[macro_export]
macro_rules! trace_default {
    ($($arg:tt)*) => {
        $crate::trace!("lycrex", $($arg)*);
    };
}

#[macro_export]
macro_rules! debug_default {
    ($($arg:tt)*) => {
        $crate::debug!("lycrex", $($arg)*);
    };
}

#[macro_export]
macro_rules! info_default {
    ($($arg:tt)*) => {
        $crate::info!("lycrex", $($arg)*);
    };
}

#[macro_export]
macro_rules! warn_default {
    ($($arg:tt)*) => {
        $crate::warn!("lycrex", $($arg)*);
    };
}

#[macro_export]
macro_rules! error_default {
    ($($arg:tt)*) => {
        $crate::error!("lycrex", $($arg)*);
    };
}

pub fn start_log_simple(level: &str, write_file: bool, time_format_int: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = LogConfig::default();

    match level {
        "trace" => {
            config.level = Level::Trace;
        }
        "debug" => {
            config.level = Level::Debug;
        }
        "info" => {
            config.level = Level::Info;
        }
        "warn" => {
            config.level = Level::Warn;
        }
        "error" => {
            config.level = Level::Error;
        }
        _ => {
            config.level = Level::Info;
        }
    }

    // 设置时间格式
    config.time_format = match time_format_int {
        0 => TimeFormat::Unix,
        1 => TimeFormat::UnixMillis,
        2 => TimeFormat::LocalTime,
        3 => TimeFormat::SystemTime,
        4 => TimeFormat::Iso8601,
        5 => TimeFormat::Relative,
        _ => TimeFormat::SystemTime,
    };

    // 文件输出器
    if write_file {
        if let Ok(file_writer) = FileWriter::new("app.log".to_string()) {
            config.writers.push(Box::new(file_writer));
        }
    }

    // 控制台输出器
    let console_formatter = DefaultFormatter {
        use_colors: true,
        show_timestamp: true,
        show_target: true,
        show_location: false,
        time_format: config.time_format,
        uptime_level: -1,
    };
    let console_writer = ConsoleWriter::with_formatter(Box::new(console_formatter));
    config.writers.push(Box::new(console_writer));
    
    init_with_config(config)
}