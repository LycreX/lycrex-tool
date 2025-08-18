use std::{
    fmt::{self, Write},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
    collections::HashMap,
};
use crate::utils::time::{TimeFormat, TimeUtils};

/// 预定义的日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PredefinedLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// 日志级别（支持自定义级别）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Level {
    Predefined(PredefinedLevel),
    Custom { name: String, priority: u8, color: String },
}

impl Level {
    /// 创建自定义级别
    pub fn custom(name: &str, priority: u8, color: &str) -> Self {
        Level::Custom {
            name: name.to_string(),
            priority,
            color: color.to_string(),
        }
    }

    /// 获取级别名称
    pub fn as_str(&self) -> String {
        match self {
            Level::Predefined(level) => match level {
                PredefinedLevel::Trace => "TRACE".to_string(),
                PredefinedLevel::Debug => "DEBUG".to_string(),
                PredefinedLevel::Info => "INFO".to_string(),
                PredefinedLevel::Warn => "WARN".to_string(),
                PredefinedLevel::Error => "ERROR".to_string(),
            },
            Level::Custom { name, .. } => name.to_uppercase(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => Level::Predefined(PredefinedLevel::Trace),
            "DEBUG" => Level::Predefined(PredefinedLevel::Debug),
            "INFO" => Level::Predefined(PredefinedLevel::Info),
            "WARN" => Level::Predefined(PredefinedLevel::Warn),
            "ERROR" => Level::Predefined(PredefinedLevel::Error),
            _ => Level::Custom { name: s.to_string(), priority: 0, color: "".to_string() },
        }
    }

    /// 获取级别优先级
    pub fn priority(&self) -> u8 {
        match self {
            Level::Predefined(level) => *level as u8,
            Level::Custom { priority, .. } => *priority,
        }
    }

    /// 获取颜色代码
    pub fn color_code(&self) -> String {
        match self {
            Level::Predefined(level) => match level {
                PredefinedLevel::Trace => "\x1b[90m".to_string(), // 灰
                PredefinedLevel::Debug => "\x1b[36m".to_string(), // 青
                PredefinedLevel::Info => "\x1b[32m".to_string(),  // 绿
                PredefinedLevel::Warn => "\x1b[33m".to_string(),  // 黄
                PredefinedLevel::Error => "\x1b[31m".to_string(), // 红
            },
            Level::Custom { color, .. } => color.clone(),
        }
    }

    /// 预定义级别便捷方法
    pub fn trace() -> Self { Level::Predefined(PredefinedLevel::Trace) }
    pub fn debug() -> Self { Level::Predefined(PredefinedLevel::Debug) }
    pub fn info() -> Self { Level::Predefined(PredefinedLevel::Info) }
    pub fn warn() -> Self { Level::Predefined(PredefinedLevel::Warn) }
    pub fn error() -> Self { Level::Predefined(PredefinedLevel::Error) }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 级别过滤器
#[derive(Debug, Clone)]
pub struct LevelFilter {
    min_level: Level,
    enabled_levels: HashMap<String, bool>,
    disabled_levels: HashMap<String, bool>,
}

impl LevelFilter {
    pub fn new(min_level: Level) -> Self {
        Self {
            min_level,
            enabled_levels: HashMap::new(),
            disabled_levels: HashMap::new(),
        }
    }

    /// 启用特定级别
    pub fn enable_level(&mut self, level_name: &str) {
        self.enabled_levels.insert(level_name.to_uppercase(), true);
        self.disabled_levels.remove(&level_name.to_uppercase());
    }

    /// 禁用特定级别
    pub fn disable_level(&mut self, level_name: &str) {
        self.disabled_levels.insert(level_name.to_uppercase(), true);
        self.enabled_levels.remove(&level_name.to_uppercase());
    }

    /// 检查级别是否应该被记录
    pub fn should_log(&self, level: &Level) -> bool {
        let level_name = level.as_str();
        
        // 检查是否被明确禁用
        if self.disabled_levels.contains_key(&level_name) {
            return false;
        }

        // 检查是否被明确启用
        if self.enabled_levels.contains_key(&level_name) {
            return true;
        }

        // 否则按照最小级别检查
        level >= &self.min_level
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
    pub level_filter: LevelFilter,
    pub writers: Vec<Box<dyn Writer>>,
    pub time_format: TimeFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level_filter: LevelFilter::new(Level::info()),
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

    /// 设置最小日志级别
    pub fn set_min_level(&mut self, level: Level) {
        self.config.level_filter = LevelFilter::new(level);
    }

    /// 启用特定级别
    pub fn enable_level(&mut self, level_name: &str) {
        self.config.level_filter.enable_level(level_name);
    }

    /// 禁用特定级别
    pub fn disable_level(&mut self, level_name: &str) {
        self.config.level_filter.disable_level(level_name);
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
        if !self.config.level_filter.should_log(&level) {
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

/// 日志级别管理器
pub struct LevelManager {
    custom_levels: HashMap<String, Level>,
}

impl LevelManager {
    pub fn new() -> Self {
        Self {
            custom_levels: HashMap::new(),
        }
    }

    /// 注册自定义级别
    pub fn register_level(&mut self, name: &str, priority: u8, color: &str) {
        let level = Level::custom(name, priority, color);
        self.custom_levels.insert(name.to_uppercase(), level);
    }

    /// 获取已注册的自定义级别
    pub fn get_level(&self, name: &str) -> Option<&Level> {
        self.custom_levels.get(&name.to_uppercase())
    }

    /// 列出所有自定义级别
    pub fn list_custom_levels(&self) -> Vec<&String> {
        self.custom_levels.keys().collect()
    }
}

/// 全局级别管理器
static LEVEL_MANAGER: Mutex<Option<LevelManager>> = Mutex::new(None);

/// 获取或初始化全局级别管理器
fn get_level_manager() -> std::sync::MutexGuard<'static, Option<LevelManager>> {
    let mut manager = LEVEL_MANAGER.lock().unwrap();
    if manager.is_none() {
        *manager = Some(LevelManager::new());
    }
    manager
}

/// 注册全局自定义级别
pub fn register_global_level(name: &str, priority: u8, color: &str) {
    let mut manager_guard = get_level_manager();
    if let Some(ref mut manager) = manager_guard.as_mut() {
        manager.register_level(name, priority, color);
    }
}

/// 获取全局自定义级别
pub fn get_global_level(name: &str) -> Option<Level> {
    let manager_guard = get_level_manager();
    if let Some(ref manager) = manager_guard.as_ref() {
        manager.get_level(name).cloned()
    } else {
        None
    }
}

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
        $crate::log!($crate::lycrex::logger::Level::trace(), $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::debug(), $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::info(), $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::warn(), $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::error(), $target, $($arg)*);
    };
}

/// 网络日志宏
#[macro_export]
macro_rules! network {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::network(), $target, $($arg)*);
    };
}

/// 自定义级别日志宏
#[macro_export]
macro_rules! custom_log {
    ($level_name:expr, $priority:expr, $color:expr, $target:expr, $($arg:tt)*) => {
        $crate::log!(
            $crate::lycrex::logger::Level::custom($level_name, $priority, $color),
            $target,
            $($arg)*
        );
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

#[macro_export]
macro_rules! network_default {
    ($($arg:tt)*) => {
        $crate::network!("lycrex", $($arg)*);
    };
}

pub fn start_log_simple(level: &str, write_file: bool, time_format_int: i8) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = LogConfig::default();

    let log_level = match level {
        "trace" => Level::trace(),
        "debug" => Level::debug(),
        "info" => Level::info(),
        "warn" => Level::warn(),
        "error" => Level::error(),
        _ => Level::info(),
    };
    
    config.level_filter = LevelFilter::new(log_level);

    // 设置时间格式
    config.time_format = TimeFormat::from_int(time_format_int).unwrap();

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

/// 高级日志记录器扩展功能
impl Logger {
    /// 获取当前级别过滤器的引用
    pub fn level_filter(&self) -> &LevelFilter {
        &self.config.level_filter
    }

    /// 获取可变的级别过滤器引用
    pub fn level_filter_mut(&mut self) -> &mut LevelFilter {
        &mut self.config.level_filter
    }

    /// 批量启用级别
    pub fn enable_levels(&mut self, level_names: &[&str]) {
        for name in level_names {
            self.enable_level(name);
        }
    }

    /// 批量禁用级别
    pub fn disable_levels(&mut self, level_names: &[&str]) {
        for name in level_names {
            self.disable_level(name);
        }
    }

    /// 检查级别是否启用
    pub fn is_level_enabled(&self, level: &Level) -> bool {
        self.config.level_filter.should_log(level)
    }

    /// 重置级别过滤器
    pub fn reset_level_filter(&mut self, min_level: Level) {
        self.config.level_filter = LevelFilter::new(min_level);
    }

    /// 使用自定义级别记录日志
    pub fn log_custom(&self, level_name: &str, priority: u8, color: &str, 
                      target: &str, message: &str, 
                      file: Option<&str>, line: Option<u32>, module_path: Option<&str>) {
        let level = Level::custom(level_name, priority, color);
        self.log(level, target, message, file, line, module_path);
    }
}

/// 全局级别控制函数
pub fn enable_global_level(_level_name: &str) {
    if let Some(_logger) = get_logger() {
        // TODO: 重构Logger架构
        println!("This function is not implemented yet");
    }
}

pub fn disable_global_level(_level_name: &str) {
    if let Some(_logger) = get_logger() {
        // TODO: 重构Logger架构
        println!("This function is not implemented yet");
    }
}