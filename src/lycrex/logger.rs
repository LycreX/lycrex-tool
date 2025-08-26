use crate::utils::time::{TimeFormat, TimeUtils};
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    fmt::{self, Write},
    sync::{Arc, Mutex, RwLock, atomic::{AtomicU64, Ordering}},
    time::{SystemTime, UNIX_EPOCH, Duration, Instant},
    io::{self, Write as IoWrite, BufWriter},
    fs::{File, OpenOptions, rename, remove_file},
    path::Path,
    thread,
    sync::mpsc::{self, Sender},
    net::{TcpStream, UdpSocket, SocketAddr},
    env,
    str::FromStr,
};

/// 预定义的日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PredefinedLevel {
    Trace = 5,
    Debug = 10,
    Info = 20,
    Notice = 30,
    Warn = 40,
    Error = 50,
    Fatal = 65,
    Record = u8::MAX as isize,
}

/// 日志级别（支持自定义级别）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Level {
    Predefined(PredefinedLevel),
    Custom {
        name: String,
        priority: u8,
        color: String,
    },
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
                PredefinedLevel::Notice => "NOTICE".to_string(),
                PredefinedLevel::Info => "INFO".to_string(),
                PredefinedLevel::Warn => "WARN".to_string(),
                PredefinedLevel::Error => "ERROR".to_string(),
                PredefinedLevel::Fatal => "FATAL".to_string(),
                PredefinedLevel::Record => "RECORD".to_string(),
            },
            Level::Custom { name, .. } => name.to_uppercase(),
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
                PredefinedLevel::Notice => "\x1b[34m".to_string(), // 蓝
                PredefinedLevel::Info => "\x1b[32m".to_string(),  // 绿
                PredefinedLevel::Warn => "\x1b[33m".to_string(),  // 黄
                PredefinedLevel::Error => "\x1b[31m".to_string(), // 红
                PredefinedLevel::Fatal => "\x1b[91;1m".to_string(), // 血红
                PredefinedLevel::Record => "".to_string(),
            },
            Level::Custom { color, .. } => color.clone(),
        }
    }

    /// 预定义级别便捷方法
    pub fn record() -> Self { Level::Predefined(PredefinedLevel::Record) }
    pub fn trace() -> Self { Level::Predefined(PredefinedLevel::Trace) }
    pub fn debug() -> Self { Level::Predefined(PredefinedLevel::Debug) }
    pub fn notice() -> Self { Level::Predefined(PredefinedLevel::Notice) }
    pub fn info() -> Self { Level::Predefined(PredefinedLevel::Info) }
    pub fn warn() -> Self { Level::Predefined(PredefinedLevel::Warn) }
    pub fn error() -> Self { Level::Predefined(PredefinedLevel::Error) }
    pub fn fatal() -> Self { Level::Predefined(PredefinedLevel::Fatal) }
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

/// 全局级别注册表
static GLOBAL_LEVEL_REGISTRY: std::sync::LazyLock<RwLock<HashMap<String, Level>>> = 
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// 注册全局自定义级别
pub fn register_level(name: &str, priority: u8, color: &str) -> Result<(), String> {
    let level_name = name.to_uppercase();
    
    // 检查是否与预定义级别冲突
    match level_name.as_str() {
        "TRACE" | "DEBUG" | "NOTICE" | "INFO" | "WARN" | "ERROR" | "FATAL" | "RECORD" => {
            return Err(format!("Cannot register level '{level_name}': conflicts with predefined level"));
        }
        _ => {}
    }
    
    let level = Level::Custom {
        name: level_name.clone(),
        priority,
        color: color.to_string(),
    };
    
    let mut registry = GLOBAL_LEVEL_REGISTRY.write().unwrap();
    registry.insert(level_name, level);
    
    Ok(())
}

/// 注销全局自定义级别
pub fn unregister_level(name: &str) -> Result<(), String> {
    let level_name = name.to_uppercase();
    
    // 检查是否为预定义级别
    match level_name.as_str() {
        "TRACE" | "DEBUG" | "NOTICE" | "INFO" | "WARN" | "ERROR" | "FATAL" | "RECORD" => {
            return Err(format!("Cannot unregister predefined level '{level_name}'"));
        }
        _ => {}
    }
    
    let mut registry = GLOBAL_LEVEL_REGISTRY.write().unwrap();
    match registry.remove(&level_name) {
        Some(_) => Ok(()),
        None => Err(format!("Level '{level_name}' not found in registry")),
    }
}

/// 获取所有注册的自定义级别
pub fn get_registered_levels() -> HashMap<String, Level> {
    let registry = GLOBAL_LEVEL_REGISTRY.read().unwrap();
    registry.clone()
}

/// 清除所有注册的自定义级别
pub fn clear_registered_levels() {
    let mut registry = GLOBAL_LEVEL_REGISTRY.write().unwrap();
    registry.clear();
}

/// 检查级别是否已注册
pub fn is_level_registered(name: &str) -> bool {
    let level_name = name.to_uppercase();
    let registry = GLOBAL_LEVEL_REGISTRY.read().unwrap();
    registry.contains_key(&level_name)
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let level_name = s.to_uppercase();
        
        // 首先检查预定义级别
        let level = match level_name.as_str() {
            "TRACE" => Level::Predefined(PredefinedLevel::Trace),
            "DEBUG" => Level::Predefined(PredefinedLevel::Debug),
            "NOTICE" => Level::Predefined(PredefinedLevel::Notice),
            "INFO" => Level::Predefined(PredefinedLevel::Info),
            "WARN" => Level::Predefined(PredefinedLevel::Warn),
            "ERROR" => Level::Predefined(PredefinedLevel::Error),
            "FATAL" => Level::Predefined(PredefinedLevel::Fatal),
            "RECORD" => Level::Predefined(PredefinedLevel::Record),
            _ => {
                // 检查全局注册表中的自定义级别
                let registry = GLOBAL_LEVEL_REGISTRY.read().unwrap();
                if let Some(custom_level) = registry.get(&level_name) {
                    custom_level.clone()
                } else {
                    // 如果未找到，创建默认的自定义级别
                    Level::Custom {
                        name: level_name,
                        priority: 30,
                        color: "".to_string(),
                    }
                }
            }
        };
        Ok(level)
    }
}



/// 级别过滤器 - 更强大的过滤系统
#[derive(Debug)]
pub struct LevelFilter {
    min_level: u8,
    enabled_levels: HashMap<String, bool>,
    disabled_levels: HashMap<String, bool>,
}

impl LevelFilter {
    pub fn new(min_level: Level) -> Self {
        Self {
            min_level: min_level.priority(),
            enabled_levels: HashMap::new(),
            disabled_levels: HashMap::new(),
        }
    }

    pub fn new_with_level(min_level: u8) -> Self {
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

    /// 设置最小级别
    pub fn set_min_level(&mut self, min_level: u8) {
        self.min_level = min_level;
    }

    pub fn set_min_level_with_level(&mut self, min_level: Level) {
        self.min_level = min_level.priority();
    }

    /// 获取最小级别
    pub fn get_min_level(&self) -> u8 {
        self.min_level
    }

    /// 批量操作
    pub fn enable_levels(&mut self, level_names: &[&str]) {
        for name in level_names {
            self.enable_level(name);
        }
    }

    pub fn disable_levels(&mut self, level_names: &[&str]) {
        for name in level_names {
            self.disable_level(name);
        }
    }

    /// 检查级别是否应该被记录
    pub fn should_log(&self, level: &Level) -> bool {
        // Record级别总是被记录
        if level == &Level::Predefined(PredefinedLevel::Record) {
            return true;
        }

        let level_name = level.as_str();

        // 检查是否被明确禁用
        if self.disabled_levels.contains_key(&level_name) {
            return false;
        }

        // 检查是否被明确启用
        if self.enabled_levels.contains_key(&level_name) {
            return true;
        }

        // 未来可以添加自定义过滤器支持

        // 否则按照最小级别检查
        level.priority() >= self.min_level
    }

    /// 重置所有级别设置
    pub fn reset_level_settings(&mut self) {
        self.enabled_levels.clear();
        self.disabled_levels.clear();
    }
}

/// 日志记录结构 - 增强版
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub timestamp: u64,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub module_path: Option<String>,
    pub thread_id: Option<String>,
    pub thread_name: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl LogRecord {
    pub fn new(level: Level, target: &str, message: &str) -> Self {
        let current_thread = thread::current();
        Self {
            level,
            target: target.to_string(),
            message: message.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            file: None,
            line: None,
            module_path: None,
            thread_id: Some(format!("{:?}", current_thread.id())),
            thread_name: current_thread.name().map(|s| s.to_string()),
            metadata: HashMap::new(),
        }
    }

    pub fn with_location(mut self, file: &str, line: u32, module_path: &str) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self.module_path = Some(module_path.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// 颜色处理工具
pub struct ColorProcessor;

impl ColorProcessor {
    /// 移除所有ANSI转义序列 - 简单实现
    pub fn strip_ansi_codes(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // 跳过ANSI转义序列
                if chars.peek() == Some(&'[') {
                    chars.next(); // 跳过 '['
                    // 继续跳过直到找到字母（转义序列的结束）
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        if next_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// 检测文本中是否包含颜色代码
    pub fn has_colors(text: &str) -> bool {
        text.contains("\x1b[")
    }

    /// 为文本添加颜色
    pub fn colorize(text: &str, color_code: &str) -> String {
        format!("{color_code}{text}\x1b[0m")
    }
}

/// 日志格式化器 - 增强版
pub trait Formatter: Send + Sync {
    fn format(&self, record: &LogRecord) -> String;
    fn supports_colors(&self) -> bool { true }
}

/// 默认格式化器 - 功能更强大
pub struct DefaultFormatter {
    pub use_colors: bool,
    pub show_timestamp: bool,
    pub timestamp_color: String,
    pub timestamp_format: String,
    pub timestamp_brackets: bool,    // 控制时间戳是否显示方括号
    pub show_level: bool,
    pub level_width: usize,
    pub level_align_right: bool,
    pub level_brackets: bool,        // 控制级别是否显示方括号
    pub show_target: bool,
    pub target_brackets: bool,       // 控制目标（事件）是否显示方括号
    pub show_location: bool,
    pub show_thread: bool,
    pub show_metadata: bool,
    pub time_format: TimeFormat,
    pub uptime_level: i8,
    pub custom_format: Option<String>,
}

impl Default for DefaultFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultFormatter {
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_timestamp: true,
            timestamp_color: "\x1b[90m".to_string(), // 灰色
            timestamp_format: "[{}]".to_string(),
            timestamp_brackets: true,    // 默认显示时间戳方括号
            show_level: true,
            level_width: 6,
            level_align_right: false,
            level_brackets: true,        // 默认显示级别方括号
            show_target: true,
            target_brackets: true,       // 默认显示目标方括号
            show_location: false,
            show_thread: false,
            show_metadata: false,
            time_format: TimeFormat::LocalTime,
            uptime_level: -1,
            custom_format: None,
        }
    }

    pub fn without_colors() -> Self {
        let mut formatter = Self::new();
        formatter.use_colors = false;
        formatter.timestamp_color = "".to_string();
        formatter
    }

    pub fn compact() -> Self {
        let mut formatter = Self::new();
        formatter.show_target = false;
        formatter.show_location = false;
        formatter.level_width = 4;
        formatter
    }

    pub fn detailed() -> Self {
        let mut formatter = Self::new();
        formatter.show_location = true;
        formatter.show_thread = true;
        formatter.show_metadata = true;
        formatter
    }
}

impl Formatter for DefaultFormatter {
    fn format(&self, record: &LogRecord) -> String {
        // 如果是Record级别，直接返回消息
        if record.level == Level::Predefined(PredefinedLevel::Record) {
            return record.message.clone();
        }

        // 如果有自定义格式，使用自定义格式
        if let Some(ref format) = self.custom_format {
            return self.format_custom(record, format);
        }

        let mut output = String::new();

        // 时间戳
        if self.show_timestamp {
            let time_str = self.format_timestamp(record);
            let formatted_time = if self.timestamp_brackets {
                self.timestamp_format.replace("{}", &time_str)
            } else {
                // 如果不显示方括号，移除timestamp_format中的方括号
                let format = self.timestamp_format.replace("[", "").replace("]", "");
                format.replace("{}", &time_str)
            };
            
            if self.use_colors && !self.timestamp_color.is_empty() {
                write!(
                    &mut output,
                    "{}{}\x1b[0m ",
                    self.timestamp_color, formatted_time
                ).unwrap();
            } else {
                write!(&mut output, "{formatted_time} ").unwrap();
            }
        }

        // 级别
        if self.show_level {
            self.format_level(&mut output, record);
        }

        // 线程信息
        if self.show_thread {
            if let Some(ref thread_name) = record.thread_name {
                write!(&mut output, "[{thread_name}] ").unwrap();
            } else if let Some(ref thread_id) = record.thread_id {
                write!(&mut output, "[{thread_id}] ").unwrap();
            }
        }

        // 目标
        if self.show_target && !record.target.is_empty() {
            if self.target_brackets {
                write!(&mut output, "({}) ", record.target).unwrap();
            } else {
                write!(&mut output, "{} ", record.target).unwrap();
            }
        }

        // 位置
        if self.show_location {
            if let (Some(ref file), Some(line)) = (&record.file, record.line) {
                let filename = Path::new(file).file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file);
                write!(&mut output, "{filename}:{line} ").unwrap();
            }
        }

        // 消息
        output.push_str(&record.message);

        // 元数据
        if self.show_metadata && !record.metadata.is_empty() {
            write!(&mut output, " ").unwrap();
            for (key, value) in &record.metadata {
                write!(&mut output, "{key}={value} ").unwrap();
            }
        }

        output
    }

    fn supports_colors(&self) -> bool {
        self.use_colors
    }
}

impl DefaultFormatter {
    fn format_timestamp(&self, record: &LogRecord) -> String {
        match self.time_format {
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
                    format!("+{uptime} {unit}")
                }
            }
        }
    }

    fn format_level(&self, output: &mut String, record: &LogRecord) {
        let level_str = if self.level_align_right {
            format!("{:>width$}", record.level.as_str(), width = self.level_width)
        } else {
            format!("{:<width$}", record.level.as_str(), width = self.level_width)
        };

        if self.use_colors {
            if self.level_brackets {
                write!(
                    output,
                    "[{}{}]\x1b[0m ",
                    record.level.color_code(),
                    level_str
                ).unwrap();
            } else {
                write!(
                    output,
                    "{}{}\x1b[0m ",
                    record.level.color_code(),
                    level_str
                ).unwrap();
            }
        } else if self.level_brackets {
            write!(output, "[{level_str}] ").unwrap();
        } else {
            write!(output, "{level_str} ").unwrap();
        }
    }

    fn format_custom(&self, record: &LogRecord, format: &str) -> String {
        let mut result = format.to_string();
        
        // 替换占位符
        result = result.replace("{timestamp}", &self.format_timestamp(record));
        result = result.replace("{level}", &record.level.as_str());
        result = result.replace("{target}", &record.target);
        result = result.replace("{message}", &record.message);
        
        if let Some(ref file) = record.file {
            result = result.replace("{file}", file);
        }
        
        if let Some(line) = record.line {
            result = result.replace("{line}", &line.to_string());
        }
        
        if let Some(ref module) = record.module_path {
            result = result.replace("{module}", module);
        }

        result
    }
}

/// JSON格式化器
pub struct JsonFormatter {
    pub pretty_print: bool,
    pub include_metadata: bool,
    pub custom_fields: HashMap<String, String>,
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {
            pretty_print: false,
            include_metadata: true,
            custom_fields: HashMap::new(),
        }
    }

    pub fn pretty(mut self) -> Self {
        self.pretty_print = true;
        self
    }

    pub fn with_custom_field(mut self, key: &str, value: &str) -> Self {
        self.custom_fields.insert(key.to_string(), value.to_string());
        self
    }

    fn escape_json_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut json_obj = Vec::new();
        
        json_obj.push(format!("\"timestamp\":{}", record.timestamp));
        json_obj.push(format!("\"level\":\"{}\"", record.level.as_str()));
        json_obj.push(format!("\"target\":\"{}\"", Self::escape_json_string(&record.target)));
        json_obj.push(format!("\"message\":\"{}\"", Self::escape_json_string(&record.message)));
        
        if let Some(ref file) = record.file {
            json_obj.push(format!("\"file\":\"{}\"", Self::escape_json_string(file)));
        }
        
        if let Some(line) = record.line {
            json_obj.push(format!("\"line\":{line}"));
        }
        
        if let Some(ref module) = record.module_path {
            json_obj.push(format!("\"module\":\"{}\"", Self::escape_json_string(module)));
        }

        if let Some(ref thread_name) = record.thread_name {
            json_obj.push(format!("\"thread_name\":\"{}\"", Self::escape_json_string(thread_name)));
        }

        if let Some(ref thread_id) = record.thread_id {
            json_obj.push(format!("\"thread_id\":\"{}\"", Self::escape_json_string(thread_id)));
        }

        // 添加自定义字段
        for (key, value) in &self.custom_fields {
            json_obj.push(format!("\"{}\":\"{}\"", 
                Self::escape_json_string(key), 
                Self::escape_json_string(value)
            ));
        }

        // 添加元数据
        if self.include_metadata && !record.metadata.is_empty() {
            let metadata_items: Vec<String> = record.metadata.iter()
                .map(|(k, v)| format!("\"{}\":\"{}\"", 
                    Self::escape_json_string(k), 
                    Self::escape_json_string(v)
                ))
                .collect();
            json_obj.push(format!("\"metadata\":{{{}}}", metadata_items.join(",")));
        }

        if self.pretty_print {
            format!("{{\n  {}\n}}", json_obj.join(",\n  "))
        } else {
            format!("{{{}}}", json_obj.join(","))
        }
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// XML格式化器
pub struct XmlFormatter {
    pub pretty_print: bool,
    pub include_metadata: bool,
    pub root_element: String,
    pub custom_attributes: HashMap<String, String>,
}

impl Default for XmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlFormatter {
    pub fn new() -> Self {
        Self {
            pretty_print: false,
            include_metadata: true,
            root_element: "log".to_string(),
            custom_attributes: HashMap::new(),
        }
    }

    pub fn pretty(mut self) -> Self {
        self.pretty_print = true;
        self
    }

    pub fn root_element(mut self, element: &str) -> Self {
        self.root_element = element.to_string();
        self
    }

    pub fn with_custom_attribute(mut self, key: &str, value: &str) -> Self {
        self.custom_attributes.insert(key.to_string(), value.to_string());
        self
    }

    fn escape_xml(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&apos;".to_string(),
                c => c.to_string(),
            })
            .collect()
    }
}

impl Formatter for XmlFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut xml = String::new();
        let indent = if self.pretty_print { "  " } else { "" };
        let newline = if self.pretty_print { "\n" } else { "" };

        // 开始标签
        xml.push_str(&format!("<{}", self.root_element));
        
        // 添加属性
        xml.push_str(&format!(" timestamp=\"{}\"", record.timestamp));
        xml.push_str(&format!(" level=\"{}\"", record.level.as_str()));
        
        for (key, value) in &self.custom_attributes {
            xml.push_str(&format!(" {}=\"{}\"", key, Self::escape_xml(value)));
        }
        
        xml.push('>');
        xml.push_str(newline);

        // 基本信息
        xml.push_str(&format!("{}<target>{}</target>{}", 
            indent, Self::escape_xml(&record.target), newline));
        xml.push_str(&format!("{}<message>{}</message>{}", 
            indent, Self::escape_xml(&record.message), newline));

        // 可选信息
        if let Some(ref file) = record.file {
            xml.push_str(&format!("{}<file>{}</file>{}", 
                indent, Self::escape_xml(file), newline));
        }
        
        if let Some(line) = record.line {
            xml.push_str(&format!("{indent}<line>{line}</line>{newline}"));
        }
        
        if let Some(ref module) = record.module_path {
            xml.push_str(&format!("{}<module>{}</module>{}", 
                indent, Self::escape_xml(module), newline));
        }

        if let Some(ref thread_name) = record.thread_name {
            xml.push_str(&format!("{}<thread_name>{}</thread_name>{}", 
                indent, Self::escape_xml(thread_name), newline));
        }

        // 元数据
        if self.include_metadata && !record.metadata.is_empty() {
            xml.push_str(&format!("{indent}<metadata>{newline}"));
            for (key, value) in &record.metadata {
                xml.push_str(&format!("{}  <{}>{}</{}>{}", 
                    indent, Self::escape_xml(key), Self::escape_xml(value), Self::escape_xml(key), newline));
            }
            xml.push_str(&format!("{indent}</metadata>{newline}"));
        }

        // 结束标签
        xml.push_str(&format!("</{}>", self.root_element));

        xml
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// 简单的结构化格式化器
pub struct StructuredFormatter {
    pub include_metadata: bool,
    pub field_separator: String,
}

impl Default for StructuredFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuredFormatter {
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            field_separator: " | ".to_string(),
        }
    }

    pub fn with_separator(separator: &str) -> Self {
        Self {
            include_metadata: true,
            field_separator: separator.to_string(),
        }
    }
}

impl Formatter for StructuredFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut parts = Vec::new();
        
        parts.push(format!("timestamp={}", record.timestamp));
        parts.push(format!("level={}", record.level.as_str()));
        parts.push(format!("target={}", record.target));
        parts.push(format!("message={}", record.message));
        
        if let Some(ref file) = record.file {
            parts.push(format!("file={file}"));
        }
        
        if let Some(line) = record.line {
            parts.push(format!("line={line}"));
        }
        
        if let Some(ref module) = record.module_path {
            parts.push(format!("module={module}"));
        }

        if self.include_metadata && !record.metadata.is_empty() {
            for (key, value) in &record.metadata {
                parts.push(format!("{key}={value}"));
            }
        }

        parts.join(&self.field_separator)
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// 日志输出器 - 增强版
pub trait Writer: Send + Sync {
    fn write(&self, record: &LogRecord);
    fn flush(&self) -> io::Result<()> { Ok(()) }
    fn as_any(&self) -> &dyn Any;
    fn supports_colors(&self) -> bool { true }
}

/// 控制台输出器 - 增强版
pub struct ConsoleWriter {
    formatter: Box<dyn Formatter>,
    use_stderr_for_errors: bool,
    color_support: bool,
}

impl Default for ConsoleWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {
            formatter: Box::new(DefaultFormatter::new()),
            use_stderr_for_errors: true,
            color_support: true,
        }
    }

    pub fn with_formatter(formatter: Box<dyn Formatter>) -> Self {
        Self {
            formatter,
            use_stderr_for_errors: true,
            color_support: true,
        }
    }

    pub fn without_colors() -> Self {
        Self {
            formatter: Box::new(DefaultFormatter::without_colors()),
            use_stderr_for_errors: true,
            color_support: false,
        }
    }

    pub fn stderr_for_errors(mut self, use_stderr: bool) -> Self {
        self.use_stderr_for_errors = use_stderr;
        self
    }

    pub fn with_color_support(mut self, color_support: bool) -> Self {
        self.color_support = color_support;
        self
    }
}

impl Writer for ConsoleWriter {
    fn write(&self, record: &LogRecord) {
        let mut message = self.formatter.format(record);
        
        // 如果选择不支持颜色，移除颜色代码
        if !self.color_support || !self.formatter.supports_colors() {
            message = ColorProcessor::strip_ansi_codes(&message);
        }

        // 根据级别选择输出流
        if self.use_stderr_for_errors && record.level.priority() >= Level::error().priority() {
            eprintln!("{message}");
        } else {
            println!("{message}");
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn supports_colors(&self) -> bool {
        self.color_support
    }
}

/// 日志轮转策略
#[derive(Debug, Clone)]
pub enum RotationPolicy {
    /// 按文件大小轮转（字节）
    Size(u64),
    /// 按时间轮转
    Time(Duration),
    /// 按日期轮转（每天、每小时等）
    Daily,
    Hourly,
    /// 自定义轮转条件
    Custom,
    /// 不轮转
    Never,
}

/// 轮转状态
#[derive(Debug)]
pub struct RotationState {
    pub current_size: u64,
    pub creation_time: SystemTime,
    pub last_rotation: SystemTime,
    pub rotation_count: u32,
}

impl Default for RotationState {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationState {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            current_size: 0,
            creation_time: now,
            last_rotation: now,
            rotation_count: 0,
        }
    }
}

/// 文件输出器（支持轮转）
pub struct FileWriter {
    path: String,
    formatter: Box<dyn Formatter>,
    append: bool,
    buffer_size: usize,
    auto_flush: bool,
    file_handle: Arc<Mutex<Option<BufWriter<File>>>>,
    rotation_policy: RotationPolicy,
    rotation_state: Arc<Mutex<RotationState>>,
    max_backup_files: u32,
    compress_backups: bool,
}

impl FileWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        Ok(Self {
            path: path_str,
            formatter: Box::new(DefaultFormatter::without_colors()),
            append: true,
            buffer_size: 8192,
            auto_flush: true,
            file_handle: Arc::new(Mutex::new(None)),
            rotation_policy: RotationPolicy::Never,
            rotation_state: Arc::new(Mutex::new(RotationState::new())),
            max_backup_files: 5,
            compress_backups: false,
        })
    }

    pub fn with_formatter<P: AsRef<Path>>(path: P, formatter: Box<dyn Formatter>) -> io::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        Ok(Self {
            path: path_str,
            formatter,
            append: true,
            buffer_size: 8192,
            auto_flush: true,
            file_handle: Arc::new(Mutex::new(None)),
            rotation_policy: RotationPolicy::Never,
            rotation_state: Arc::new(Mutex::new(RotationState::new())),
            max_backup_files: 5,
            compress_backups: false,
        })
    }

    pub fn with_rotation<P: AsRef<Path>>(path: P, policy: RotationPolicy) -> io::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        Ok(Self {
            path: path_str,
            formatter: Box::new(DefaultFormatter::without_colors()),
            append: true,
            buffer_size: 8192,
            auto_flush: true,
            file_handle: Arc::new(Mutex::new(None)),
            rotation_policy: policy,
            rotation_state: Arc::new(Mutex::new(RotationState::new())),
            max_backup_files: 5,
            compress_backups: false,
        })
    }

    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn auto_flush(mut self, auto_flush: bool) -> Self {
        self.auto_flush = auto_flush;
        self
    }

    pub fn rotation_policy(mut self, policy: RotationPolicy) -> Self {
        self.rotation_policy = policy;
        self
    }

    pub fn max_backup_files(mut self, count: u32) -> Self {
        self.max_backup_files = count;
        self
    }

    pub fn compress_backups(mut self, compress: bool) -> Self {
        self.compress_backups = compress;
        self
    }

    #[allow(dead_code)]
    fn should_rotate(&self, _record: &LogRecord) -> bool {
        let state = self.rotation_state.lock().unwrap();
        match &self.rotation_policy {
            RotationPolicy::Never => false,
            RotationPolicy::Size(max_size) => state.current_size >= *max_size,
            RotationPolicy::Time(duration) => {
                SystemTime::now().duration_since(state.last_rotation)
                    .map(|d| d >= *duration)
                    .unwrap_or(false)
            },
            RotationPolicy::Daily => {
                let now = SystemTime::now();
                let last_rotation = state.last_rotation;
                
                // 检查是否跨天
                if let (Ok(now_duration), Ok(last_duration)) = (
                    now.duration_since(UNIX_EPOCH),
                    last_rotation.duration_since(UNIX_EPOCH)
                ) {
                    let now_days = now_duration.as_secs() / 86400;
                    let last_days = last_duration.as_secs() / 86400;
                    now_days > last_days
                } else {
                    false
                }
            },
            RotationPolicy::Hourly => {
                let now = SystemTime::now();
                let last_rotation = state.last_rotation;
                
                if let (Ok(now_duration), Ok(last_duration)) = (
                    now.duration_since(UNIX_EPOCH),
                    last_rotation.duration_since(UNIX_EPOCH)
                ) {
                    let now_hours = now_duration.as_secs() / 3600;
                    let last_hours = last_duration.as_secs() / 3600;
                    now_hours > last_hours
                } else {
                    false
                }
            },
            RotationPolicy::Custom => false, // 需要外部实现
        }
    }

    fn rotate_file(&self) -> io::Result<()> {
        // 关闭当前文件
        {
            let mut handle = self.file_handle.lock().unwrap();
            if let Some(writer) = handle.take() {
                writer.into_inner()?.sync_all()?;
            }
        }

        // 生成备份文件名
        let backup_path = self.generate_backup_path()?;
        
        // 重命名当前文件为备份文件
        if Path::new(&self.path).exists() {
            rename(&self.path, &backup_path)?;
        }

        // 清理旧的备份文件
        self.cleanup_old_backups()?;

        // 重置轮转状态
        {
            let mut state = self.rotation_state.lock().unwrap();
            state.current_size = 0;
            state.last_rotation = SystemTime::now();
            state.rotation_count += 1;
        }

        // 重新创建文件句柄以确保后续写入正常
        let _ = self.get_or_create_file();

        Ok(())
    }

    fn generate_backup_path(&self) -> io::Result<String> {
        let path = Path::new(&self.path);
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let parent = path.parent()
            .unwrap_or(Path::new("."));

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?;

        let timestamp_secs = now.as_secs();
        let timestamp_micros = now.subsec_micros();
        let timestamp = format!("{timestamp_secs}.{timestamp_micros:06}");
        
        let backup_name = if extension.is_empty() {
            format!("{stem}.{timestamp}")
        } else {
            format!("{stem}.{timestamp}.{extension}")
        };

        Ok(parent.join(backup_name).to_string_lossy().to_string())
    }

    fn cleanup_old_backups(&self) -> io::Result<()> {
        if self.max_backup_files == 0 {
            return Ok(());
        }

        let path = Path::new(&self.path);
        let parent = path.parent().unwrap_or(Path::new("."));
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");

        let mut backup_files = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(stem) && name != path.file_name().unwrap().to_str().unwrap() {
                        if let Ok(metadata) = entry.metadata() {
                            backup_files.push((entry.path(), metadata.modified().unwrap_or(UNIX_EPOCH)));
                        }
                    }
                }
            }
        }

        // 按修改时间排序，保留最新的文件
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // 删除超过限制的文件
        for (path, _) in backup_files.into_iter().skip(self.max_backup_files as usize) {
            let _ = remove_file(path);
        }

        Ok(())
    }

    fn get_or_create_file(&self) -> io::Result<()> {
        let mut handle = self.file_handle.lock().unwrap();
        if handle.is_none() {
            let mut file = if self.append {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&self.path)?
            };
            
            // 优化文件大小获取：
            // - 对于覆盖模式（truncate），文件大小为 0
            // - 对于追加模式，使用 seek 获取当前位置，避免额外的 metadata 系统调用
            let file_size = if self.append {
                use std::io::{Seek, SeekFrom};
                match file.seek(SeekFrom::End(0)) {
                    Ok(size) => size,
                    Err(_) => {
                        // 如果 seek 失败，回退到 metadata 方式
                        std::fs::metadata(&self.path)
                            .map(|m| m.len())
                            .unwrap_or(0)
                    }
                }
            } else {
                0 // 覆盖模式，文件大小为 0
            };
            
            let writer = BufWriter::with_capacity(self.buffer_size, file);
            *handle = Some(writer);

            // 更新文件大小状态
            {
                let mut state = self.rotation_state.lock().unwrap();
                state.current_size = file_size;
            }
        }
        Ok(())
    }
}

impl Writer for FileWriter {
    fn write(&self, record: &LogRecord) {
        if self.get_or_create_file().is_err() {
            return;
        }

        let mut message = self.formatter.format(record);
        
        // 文件输出总是移除颜色代码
        message = ColorProcessor::strip_ansi_codes(&message);

        let message_bytes = message.len() as u64 + 1; // +1 for newline

        // 先写入记录
        let mut handle = self.file_handle.lock().unwrap();
        if let Some(ref mut writer) = *handle {
            if writeln!(writer, "{message}").is_err() {
                return;
            }
            
            if self.auto_flush {
                let _ = writer.flush();
            }

            // 更新文件大小
            {
                let mut state = self.rotation_state.lock().unwrap();
                state.current_size += message_bytes;
            }
        }
        drop(handle); // 释放锁

        // 写入后检查是否需要轮转
        let need_rotate = {
            let state = self.rotation_state.lock().unwrap();
            match &self.rotation_policy {
                RotationPolicy::Never => false,
                RotationPolicy::Size(max_size) => {
                    state.current_size >= *max_size
                },
                RotationPolicy::Time(duration) => {
                    SystemTime::now().duration_since(state.last_rotation)
                        .map(|d| d >= *duration)
                        .unwrap_or(false)
                },
                RotationPolicy::Daily => {
                    let now = SystemTime::now();
                    let last_rotation = state.last_rotation;
                    
                    if let (Ok(now_duration), Ok(last_duration)) = (
                        now.duration_since(UNIX_EPOCH),
                        last_rotation.duration_since(UNIX_EPOCH)
                    ) {
                        let now_days = now_duration.as_secs() / 86400;
                        let last_days = last_duration.as_secs() / 86400;
                        now_days > last_days
                    } else {
                        false
                    }
                },
                RotationPolicy::Hourly => {
                    let now = SystemTime::now();
                    let last_rotation = state.last_rotation;
                    
                    if let (Ok(now_duration), Ok(last_duration)) = (
                        now.duration_since(UNIX_EPOCH),
                        last_rotation.duration_since(UNIX_EPOCH)
                    ) {
                        let now_hours = now_duration.as_secs() / 3600;
                        let last_hours = last_duration.as_secs() / 3600;
                        now_hours > last_hours
                    } else {
                        false
                    }
                },
                RotationPolicy::Custom => false,
            }
        };

        // 如果需要轮转，执行轮转
        if need_rotate {
            let _ = self.rotate_file();
        }
    }

    fn flush(&self) -> io::Result<()> {
        let mut handle = self.file_handle.lock().unwrap();
        if let Some(ref mut writer) = *handle {
            writer.flush()?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// 网络日志输出器
pub struct NetworkWriter {
    target: String,
    formatter: Box<dyn Formatter>,
    protocol: NetworkProtocol,
    #[allow(dead_code)]
    connection_pool: Arc<Mutex<Vec<Box<dyn NetworkConnection>>>>,
    max_connections: usize,
    connection_timeout: Duration,
    retry_attempts: u32,
    buffer: Arc<Mutex<VecDeque<String>>>,
    buffer_size: usize,
    #[allow(dead_code)]
    auto_flush: bool,
}

#[derive(Debug, Clone)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Http { endpoint: String, headers: HashMap<String, String> },
    Syslog { facility: u8, severity: u8 },
}

pub trait NetworkConnection: Send + Sync {
    fn send(&mut self, data: &str) -> io::Result<()>;
    fn is_connected(&self) -> bool;
    fn reconnect(&mut self) -> io::Result<()>;
    fn close(&mut self) -> io::Result<()>;
}

pub struct TcpConnection {
    stream: Option<TcpStream>,
    address: SocketAddr,
    timeout: Duration,
}

impl TcpConnection {
    pub fn new(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            stream: None,
            address,
            timeout,
        }
    }
}

impl NetworkConnection for TcpConnection {
    fn send(&mut self, data: &str) -> io::Result<()> {
        if !self.is_connected() {
            self.reconnect()?;
        }
        
        if let Some(ref mut stream) = self.stream {
            stream.write_all(data.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    fn reconnect(&mut self) -> io::Result<()> {
        let stream = TcpStream::connect_timeout(&self.address, self.timeout)?;
        self.stream = Some(stream);
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        Ok(())
    }
}

pub struct UdpConnection {
    socket: Option<UdpSocket>,
    address: SocketAddr,
}

impl UdpConnection {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            socket: None,
            address,
        }
    }
}

impl NetworkConnection for UdpConnection {
    fn send(&mut self, data: &str) -> io::Result<()> {
        if !self.is_connected() {
            self.reconnect()?;
        }
        
        if let Some(ref socket) = self.socket {
            socket.send_to(data.as_bytes(), self.address)?;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    fn reconnect(&mut self) -> io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(self.address)?;
        self.socket = Some(socket);
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        self.socket = None;
        Ok(())
    }
}

impl NetworkWriter {
    pub fn tcp(address: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            target: format!("tcp://{address}"),
            formatter: Box::new(DefaultFormatter::without_colors()),
            protocol: NetworkProtocol::Tcp,
            connection_pool: Arc::new(Mutex::new(Vec::new())),
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            retry_attempts: 3,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            buffer_size: 1000,
            auto_flush: true,
        })
    }

    pub fn udp(address: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            target: format!("udp://{address}"),
            formatter: Box::new(DefaultFormatter::without_colors()),
            protocol: NetworkProtocol::Udp,
            connection_pool: Arc::new(Mutex::new(Vec::new())),
            max_connections: 1,
            connection_timeout: Duration::from_secs(1),
            retry_attempts: 1,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            buffer_size: 1000,
            auto_flush: true,
        })
    }

    pub fn with_formatter(mut self, formatter: Box<dyn Formatter>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn max_connections(mut self, count: usize) -> Self {
        self.max_connections = count;
        self
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    fn get_connection(&self) -> Option<Box<dyn NetworkConnection>> {
        match self.protocol {
            NetworkProtocol::Tcp => {
                if let Some(addr_str) = self.target.strip_prefix("tcp://") {
                    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                        Some(Box::new(TcpConnection::new(addr, self.connection_timeout)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            NetworkProtocol::Udp => {
                if let Some(addr_str) = self.target.strip_prefix("udp://") {
                    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                        Some(Box::new(UdpConnection::new(addr)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    fn send_with_retry(&self, data: &str) -> io::Result<()> {
        for attempt in 0..self.retry_attempts {
            if let Some(mut connection) = self.get_connection() {
                match connection.send(data) {
                    Ok(()) => return Ok(()),
                    Err(e) if attempt == self.retry_attempts - 1 => return Err(e),
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
                        continue;
                    }
                }
            }
        }
        
        Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Failed to establish connection"))
    }
}

impl Writer for NetworkWriter {
    fn write(&self, record: &LogRecord) {
        let message = self.formatter.format(record);
        let clean_message = ColorProcessor::strip_ansi_codes(&message);

        // 尝试直接发送
        if self.send_with_retry(&clean_message).is_err() {
            // 发送失败，添加到缓冲区
            let mut buffer = self.buffer.lock().unwrap();
            if buffer.len() < self.buffer_size {
                buffer.push_back(clean_message);
            } else {
                // 缓冲区满了，移除最老的记录
                buffer.pop_front();
                buffer.push_back(clean_message);
            }
        }
    }

    fn flush(&self) -> io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        let messages: Vec<String> = buffer.drain(..).collect();
        drop(buffer);

        for message in messages {
            self.send_with_retry(&message)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// 系统日志输出器 (Syslog)
pub struct SyslogWriter {
    formatter: Box<dyn Formatter>,
    facility: u8,
    hostname: String,
    app_name: String,
    process_id: u32,
    socket: Option<UdpSocket>,
    server: SocketAddr,
}

impl SyslogWriter {
    pub fn new(server: SocketAddr, facility: u8) -> io::Result<Self> {
        let hostname = env::var("HOSTNAME")
            .or_else(|_| env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let app_name = env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "rust-app".to_string());

        Ok(Self {
            formatter: Box::new(DefaultFormatter::without_colors()),
            facility,
            hostname,
            app_name,
            process_id: std::process::id(),
            socket: None,
            server,
        })
    }

    pub fn with_formatter(mut self, formatter: Box<dyn Formatter>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn app_name(mut self, app_name: String) -> Self {
        self.app_name = app_name;
        self
    }

    fn level_to_severity(&self, level: &Level) -> u8 {
        match level {
            Level::Predefined(PredefinedLevel::Fatal) => 0, // Emergency
            Level::Predefined(PredefinedLevel::Error) => 3, // Error
            Level::Predefined(PredefinedLevel::Warn) => 4,  // Warning
            Level::Predefined(PredefinedLevel::Notice) => 5, // Notice
            Level::Predefined(PredefinedLevel::Info) => 6,  // Info
            Level::Predefined(PredefinedLevel::Debug) => 7, // Debug
            Level::Predefined(PredefinedLevel::Trace) => 7, // Debug
            Level::Custom { priority, .. } => {
                match *priority {
                    0..=10 => 7,    // Debug
                    11..=20 => 6,   // Info
                    21..=30 => 5,   // Notice
                    31..=40 => 4,   // Warning
                    41..=50 => 3,   // Error
                    _ => 0,         // Emergency
                }
            },
            _ => 6, // Info
        }
    }

    fn format_syslog_message(&self, record: &LogRecord) -> String {
        let priority = (self.facility << 3) | self.level_to_severity(&record.level);
        let timestamp = TimeUtils::iso8601_time_string();
        let message = ColorProcessor::strip_ansi_codes(&self.formatter.format(record));
        
        format!(
            "<{}>{} {} {}[{}]: {}",
            priority,
            timestamp,
            self.hostname,
            self.app_name,
            self.process_id,
            message
        )
    }

    fn ensure_socket(&mut self) -> io::Result<()> {
        if self.socket.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:0")?;
            socket.connect(self.server)?;
            self.socket = Some(socket);
        }
        Ok(())
    }
}

impl Writer for SyslogWriter {
    fn write(&self, record: &LogRecord) {
        let mut writer = unsafe { 
            // 这里使用unsafe来获取可变引用，实际使用中应该用Arc<Mutex<>>包装
            std::ptr::read(self as *const Self as *mut Self)
        };
        
        if writer.ensure_socket().is_err() {
            return;
        }

        let message = writer.format_syslog_message(record);
        
        if let Some(ref socket) = writer.socket {
            let _ = socket.send(message.as_bytes());
        }
        
        // 防止内存泄漏
        std::mem::forget(writer);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn supports_colors(&self) -> bool {
        false
    }
}

/// 性能监控器
#[derive(Debug)]
pub struct LoggerMetrics {
    pub total_logs: AtomicU64,
    pub logs_by_level: Arc<RwLock<HashMap<String, AtomicU64>>>,
    pub total_bytes: AtomicU64,
    pub errors: AtomicU64,
    pub start_time: Instant,
    pub last_log_time: Arc<RwLock<Option<Instant>>>,
}

impl Default for LoggerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerMetrics {
    pub fn new() -> Self {
        Self {
            total_logs: AtomicU64::new(0),
            logs_by_level: Arc::new(RwLock::new(HashMap::new())),
            total_bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            start_time: Instant::now(),
            last_log_time: Arc::new(RwLock::new(None)),
        }
    }

    pub fn record_log(&self, level: &Level, bytes: u64) {
        self.total_logs.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);

        let level_name = level.as_str();
        {
            let mut levels = self.logs_by_level.write().unwrap();
            levels.entry(level_name.clone())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        {
            let mut last_time = self.last_log_time.write().unwrap();
            *last_time = Some(Instant::now());
        }
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> LoggerStats {
        let total_logs = self.total_logs.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let uptime = self.start_time.elapsed();

        let mut level_stats = HashMap::new();
        {
            let levels = self.logs_by_level.read().unwrap();
            for (level, count) in levels.iter() {
                level_stats.insert(level.clone(), count.load(Ordering::Relaxed));
            }
        }

        let last_log_time = {
            let last_time = self.last_log_time.read().unwrap();
            *last_time
        };

        LoggerStats {
            total_logs,
            total_bytes,
            errors,
            uptime,
            logs_per_second: if uptime.as_secs() > 0 { total_logs / uptime.as_secs() } else { 0 },
            bytes_per_second: if uptime.as_secs() > 0 { total_bytes / uptime.as_secs() } else { 0 },
            level_stats,
            last_log_time,
        }
    }

    pub fn reset(&self) {
        self.total_logs.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
        
        let levels = self.logs_by_level.write().unwrap();
        for (_, count) in levels.iter() {
            count.store(0, Ordering::Relaxed);
        }

        let mut last_time = self.last_log_time.write().unwrap();
        *last_time = None;
    }
}

#[derive(Debug, Clone)]
pub struct LoggerStats {
    pub total_logs: u64,
    pub total_bytes: u64,
    pub errors: u64,
    pub uptime: Duration,
    pub logs_per_second: u64,
    pub bytes_per_second: u64,
    pub level_stats: HashMap<String, u64>,
    pub last_log_time: Option<Instant>,
}

/// 日志中间件trait
pub trait LogMiddleware: Send + Sync {
    fn before_log(&self, record: &mut LogRecord) -> bool; // 返回false则跳过日志
    fn after_log(&self, record: &LogRecord, result: &io::Result<()>);
    fn name(&self) -> &str;
}

/// 采样中间件 - 按频率采样
pub struct SamplingMiddleware {
    name: String,
    sample_rate: f64, // 0.0-1.0
    counter: AtomicU64,
}

impl SamplingMiddleware {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            name: "sampling".to_string(),
            sample_rate: sample_rate.clamp(0.0, 1.0),
            counter: AtomicU64::new(0),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

impl LogMiddleware for SamplingMiddleware {
    fn before_log(&self, _record: &mut LogRecord) -> bool {
        if self.sample_rate >= 1.0 {
            return true;
        }
        
        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        let threshold = (1.0 / self.sample_rate) as u64;
        count % threshold == 0
    }

    fn after_log(&self, _record: &LogRecord, _result: &io::Result<()>) {}

    fn name(&self) -> &str {
        &self.name
    }
}

/// 频率限制中间件
pub struct RateLimitMiddleware {
    name: String,
    max_logs_per_second: u64,
    window_size: Duration,
    log_timestamps: Arc<Mutex<VecDeque<Instant>>>,
}

impl RateLimitMiddleware {
    pub fn new(max_logs_per_second: u64) -> Self {
        Self {
            name: "rate_limit".to_string(),
            max_logs_per_second,
            window_size: Duration::from_secs(1),
            log_timestamps: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn with_window(mut self, window: Duration) -> Self {
        self.window_size = window;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

impl LogMiddleware for RateLimitMiddleware {
    fn before_log(&self, _record: &mut LogRecord) -> bool {
        let now = Instant::now();
        let mut timestamps = self.log_timestamps.lock().unwrap();
        
        // 清理过期的时间戳
        while let Some(&front_time) = timestamps.front() {
            if now.duration_since(front_time) > self.window_size {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        
        // 检查是否超过限制
        if timestamps.len() as u64 >= self.max_logs_per_second {
            return false;
        }
        
        timestamps.push_back(now);
        true
    }

    fn after_log(&self, _record: &LogRecord, _result: &io::Result<()>) {}

    fn name(&self) -> &str {
        &self.name
    }
}

/// 上下文中间件 - 添加请求ID等上下文信息
pub struct ContextMiddleware {
    name: String,
    context_data: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ContextMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMiddleware {
    pub fn new() -> Self {
        Self {
            name: "context".to_string(),
            context_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_context(&self, key: &str, value: &str) {
        let mut context = self.context_data.write().unwrap();
        context.insert(key.to_string(), value.to_string());
    }

    pub fn remove_context(&self, key: &str) {
        let mut context = self.context_data.write().unwrap();
        context.remove(key);
    }

    pub fn clear_context(&self) {
        let mut context = self.context_data.write().unwrap();
        context.clear();
    }
}

impl LogMiddleware for ContextMiddleware {
    fn before_log(&self, record: &mut LogRecord) -> bool {
        let context = self.context_data.read().unwrap();
        for (key, value) in context.iter() {
            record.metadata.insert(key.clone(), value.clone());
        }
        true
    }

    fn after_log(&self, _record: &LogRecord, _result: &io::Result<()>) {}

    fn name(&self) -> &str {
        &self.name
    }
}

/// 过滤中间件
pub struct FilterMiddleware {
    name: String,
    filter_fn: Arc<dyn Fn(&LogRecord) -> bool + Send + Sync>,
}

impl FilterMiddleware {
    pub fn new<F>(filter_fn: F) -> Self 
    where
        F: Fn(&LogRecord) -> bool + Send + Sync + 'static,
    {
        Self {
            name: "filter".to_string(),
            filter_fn: Arc::new(filter_fn),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 按级别过滤
    pub fn by_level(min_level: Level) -> Self {
        Self::new(move |record| record.level >= min_level)
    }

    /// 按目标过滤
    pub fn by_target(target_pattern: String) -> Self {
        Self::new(move |record| record.target.contains(&target_pattern))
    }

    /// 按消息内容过滤
    pub fn by_message_contains(pattern: String) -> Self {
        Self::new(move |record| record.message.contains(&pattern))
    }
}

impl LogMiddleware for FilterMiddleware {
    fn before_log(&self, record: &mut LogRecord) -> bool {
        (self.filter_fn)(record)
    }

    fn after_log(&self, _record: &LogRecord, _result: &io::Result<()>) {}

    fn name(&self) -> &str {
        &self.name
    }
}

/// 带中间件的Writer包装器
pub struct MiddlewareWriter {
    writer: Box<dyn Writer>,
    middlewares: Vec<Box<dyn LogMiddleware>>,
    metrics: Option<Arc<LoggerMetrics>>,
}

impl MiddlewareWriter {
    pub fn new(writer: Box<dyn Writer>) -> Self {
        Self {
            writer,
            middlewares: Vec::new(),
            metrics: None,
        }
    }

    pub fn with_middleware(mut self, middleware: Box<dyn LogMiddleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn with_metrics(mut self, metrics: Arc<LoggerMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

impl Writer for MiddlewareWriter {
    fn write(&self, record: &LogRecord) {
        let mut record = record.clone();
        
        // 执行前置中间件
        for middleware in &self.middlewares {
            if !middleware.before_log(&mut record) {
                // 中间件决定跳过这条日志
                return;
            }
        }

        // 记录性能指标
        if let Some(ref metrics) = self.metrics {
            let message_size = record.message.len() as u64;
            metrics.record_log(&record.level, message_size);
        }

        // 实际写入
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.writer.write(&record);
            Ok(())
        }));

        let write_result = match result {
            Ok(r) => r,
            Err(_) => {
                if let Some(ref metrics) = self.metrics {
                    metrics.record_error();
                }
                Err(io::Error::other("Writer panicked"))
            }
        };

        // 执行后置中间件
        for middleware in &self.middlewares {
            middleware.after_log(&record, &write_result);
        }
    }

    fn flush(&self) -> io::Result<()> {
        self.writer.flush()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn supports_colors(&self) -> bool {
        self.writer.supports_colors()
    }
}

/// 异步日志写入器
pub struct AsyncWriter {
    sender: Sender<LogRecord>,
    _handle: thread::JoinHandle<()>,
}

impl AsyncWriter {
    pub fn new(writer: Box<dyn Writer>) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let handle = thread::spawn(move || {
            for record in receiver {
                writer.write(&record);
            }
        });

        Self {
            sender,
            _handle: handle,
        }
    }
}

impl Writer for AsyncWriter {
    fn write(&self, record: &LogRecord) {
        let _ = self.sender.send(record.clone());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 日志配置
pub struct LogConfig {
    pub level_filter: LevelFilter,
    pub writers: Vec<Box<dyn Writer>>,
    pub time_format: TimeFormat,
    pub async_logging: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level_filter: LevelFilter::new(Level::info()),
            writers: vec![],
            time_format: TimeFormat::LocalTime,
            async_logging: false,
        }
    }
}

/// Logger构建器 - 强大的Builder模式
pub struct LoggerBuilder {
    config: LogConfig,
    console_formatter: Option<Box<dyn Formatter>>,
    file_path: Option<String>,
    file_formatter: Option<Box<dyn Formatter>>,
    file_append: bool,            // 控制文件是否追加写入
    use_colors: bool,
    show_timestamp: bool,
    show_target: bool,
    show_location: bool,
    show_thread: bool,
    level_width: usize,
    timestamp_brackets: bool,     // 控制时间戳方括号
    level_brackets: bool,         // 控制级别方括号
    target_brackets: bool,        // 控制目标方括号
    custom_writers: Vec<Box<dyn Writer>>,
    rotation_policy: Option<RotationPolicy>,
    max_backup_files: u32,
    middlewares: Vec<Box<dyn LogMiddleware>>,
    metrics: Option<Arc<LoggerMetrics>>,
    network_writers: Vec<(String, NetworkProtocol)>,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
            console_formatter: None,
            file_path: None,
            file_formatter: None,
            file_append: true,            // 默认追加写入
            use_colors: true,
            show_timestamp: true,
            show_target: true,
            show_location: false,
            show_thread: false,
            level_width: 6,
            timestamp_brackets: true,     // 默认显示时间戳方括号
            level_brackets: true,         // 默认显示级别方括号
            target_brackets: true,        // 默认显示目标方括号
            custom_writers: Vec::new(),
            rotation_policy: None,
            max_backup_files: 5,
            middlewares: Vec::new(),
            metrics: None,
            network_writers: Vec::new(),
        }
    }

    /// 设置日志级别
    pub fn level<L: Into<Level>>(mut self, level: L) -> Self {
        self.config.level_filter.set_min_level_with_level(level.into());
        self
    }

    pub fn level_str(mut self, level: &str) -> Self {
        let log_level = level.parse::<Level>().unwrap_or_else(|_| Level::Custom {
            name: level.to_string(),
            priority: 30,
            color: "".to_string(),
        });
        self.config.level_filter.set_min_level_with_level(log_level);
        self
    }

    /// 时间格式设置
    pub fn time_format(mut self, format: TimeFormat) -> Self {
        self.config.time_format = format;
        self
    }

    pub fn time_format_str(mut self, format: &str) -> Self {
        let time_format = match format.to_lowercase().as_str() {
            "unix" => TimeFormat::Unix,
            "unix_millis" => TimeFormat::UnixMillis,
            "system" => TimeFormat::SystemTime,
            "local" => TimeFormat::LocalTime,
            "iso8601" => TimeFormat::Iso8601,
            "relative" => TimeFormat::Relative,
            _ => TimeFormat::LocalTime,
        };
        self.config.time_format = time_format;
        self
    }

    /// 颜色设置
    pub fn with_colors(mut self) -> Self {
        self.use_colors = true;
        self
    }

    pub fn without_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }

    /// 显示选项
    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    pub fn show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    pub fn show_location(mut self, show: bool) -> Self {
        self.show_location = show;
        self
    }

    pub fn show_thread(mut self, show: bool) -> Self {
        self.show_thread = show;
        self
    }

    pub fn level_width(mut self, width: usize) -> Self {
        self.level_width = width;
        self
    }

    /// 设置时间戳是否显示方括号
    pub fn timestamp_brackets(mut self, show_brackets: bool) -> Self {
        self.timestamp_brackets = show_brackets;
        self
    }

    /// 设置级别是否显示方括号
    pub fn level_brackets(mut self, show_brackets: bool) -> Self {
        self.level_brackets = show_brackets;
        self
    }

    /// 设置目标（事件）是否显示方括号
    pub fn target_brackets(mut self, show_brackets: bool) -> Self {
        self.target_brackets = show_brackets;
        self
    }

    /// 设置所有元素都不显示方括号（时间戳、级别、目标）
    pub fn without_brackets(mut self) -> Self {
        self.timestamp_brackets = false;
        self.level_brackets = false;
        self.target_brackets = false;
        self
    }

    /// 设置所有元素都显示方括号（时间戳、级别、目标）
    pub fn with_brackets(mut self) -> Self {
        self.timestamp_brackets = true;
        self.level_brackets = true;
        self.target_brackets = true;
        self
    }

    /// 输出设置
    pub fn console(self) -> Self {
        // 会在build时添加console writer
        self
    }

    pub fn file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.file_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    /// 自定义格式化器
    pub fn console_formatter(mut self, formatter: Box<dyn Formatter>) -> Self {
        self.console_formatter = Some(formatter);
        self
    }

    pub fn file_formatter(mut self, formatter: Box<dyn Formatter>) -> Self {
        self.file_formatter = Some(formatter);
        self
    }

    /// 设置文件写入模式
    /// - `append`: true 表示追加写入（默认），false 表示覆盖写入
    pub fn file_append(mut self, append: bool) -> Self {
        self.file_append = append;
        self
    }

    /// 设置文件为覆盖模式（清空原有内容）
    pub fn file_overwrite(mut self) -> Self {
        self.file_append = false;
        self
    }

    /// 添加自定义Writer
    pub fn add_writer(mut self, writer: Box<dyn Writer>) -> Self {
        self.custom_writers.push(writer);
        self
    }

    /// 异步日志
    pub fn async_logging(mut self, async_log: bool) -> Self {
        self.config.async_logging = async_log;
        self
    }

    /// 高级配置
    pub fn enable_level(mut self, level_name: &str) -> Self {
        self.config.level_filter.enable_level(level_name);
        self
    }

    pub fn disable_level(mut self, level_name: &str) -> Self {
        self.config.level_filter.disable_level(level_name);
        self
    }

    pub fn enable_levels(mut self, level_names: &[&str]) -> Self {
        self.config.level_filter.enable_levels(level_names);
        self
    }

    pub fn disable_levels(mut self, level_names: &[&str]) -> Self {
        self.config.level_filter.disable_levels(level_names);
        self
    }

    /// 文件轮转设置
    pub fn file_rotation(mut self, policy: RotationPolicy) -> Self {
        self.rotation_policy = Some(policy);
        self
    }

    pub fn max_backup_files(mut self, count: u32) -> Self {
        self.max_backup_files = count;
        self
    }

    /// 网络日志输出
    pub fn tcp_output(mut self, address: &str) -> Self {
        if let Ok(addr) = address.parse::<SocketAddr>() {
            self.network_writers.push((format!("tcp://{addr}"), NetworkProtocol::Tcp));
        }
        self
    }

    pub fn udp_output(mut self, address: &str) -> Self {
        if let Ok(addr) = address.parse::<SocketAddr>() {
            self.network_writers.push((format!("udp://{addr}"), NetworkProtocol::Udp));
        }
        self
    }

    pub fn syslog_output(mut self, address: &str, facility: u8) -> Self {
        if let Ok(addr) = address.parse::<SocketAddr>() {
            self.network_writers.push((
                format!("syslog://{addr}"), 
                NetworkProtocol::Syslog { facility, severity: 6 }
            ));
        }
        self
    }

    /// 中间件设置
    pub fn with_middleware(mut self, middleware: Box<dyn LogMiddleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn with_sampling(mut self, rate: f64) -> Self {
        self.middlewares.push(Box::new(SamplingMiddleware::new(rate)));
        self
    }

    pub fn with_rate_limit(mut self, max_per_second: u64) -> Self {
        self.middlewares.push(Box::new(RateLimitMiddleware::new(max_per_second)));
        self
    }

    pub fn with_context(mut self) -> Self {
        self.middlewares.push(Box::new(ContextMiddleware::new()));
        self
    }

    pub fn with_filter<F>(mut self, filter_fn: F) -> Self 
    where
        F: Fn(&LogRecord) -> bool + Send + Sync + 'static,
    {
        self.middlewares.push(Box::new(FilterMiddleware::new(filter_fn)));
        self
    }

    /// 性能监控
    pub fn with_metrics(mut self) -> Self {
        self.metrics = Some(Arc::new(LoggerMetrics::new()));
        self
    }

    pub fn with_shared_metrics(mut self, metrics: Arc<LoggerMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// 快捷设置方法
    pub fn json_format(mut self) -> Self {
        self.console_formatter = Some(Box::new(JsonFormatter::new()));
        self.file_formatter = Some(Box::new(JsonFormatter::new()));
        self
    }

    pub fn xml_format(mut self) -> Self {
        self.console_formatter = Some(Box::new(XmlFormatter::new()));
        self.file_formatter = Some(Box::new(XmlFormatter::new()));
        self
    }

    pub fn structured_format(mut self) -> Self {
        self.console_formatter = Some(Box::new(StructuredFormatter::new()));
        self.file_formatter = Some(Box::new(StructuredFormatter::new()));
        self
    }

    /// 预设配置
    pub fn development(self) -> Self {
        self.level(Level::debug())
            .with_colors()
            .show_location(true)
            .show_thread(true)
            .console()
    }

    pub fn production(self) -> Self {
        self.level(Level::info())
            .without_colors()
            .json_format()
            .with_metrics()
            .with_sampling(0.1) // 10%采样
            .with_rate_limit(1000) // 每秒最多1000条
    }

    pub fn high_performance(self) -> Self {
        self.level(Level::warn())
            .without_colors()
            .async_logging(true)
            .with_sampling(0.01) // 1%采样
            .with_rate_limit(100)
    }

    /// 构建Logger
    pub fn build(mut self) -> Result<Logger, Box<dyn std::error::Error>> {
        // 创建控制台formatter
        let console_formatter = self.console_formatter.unwrap_or_else(|| {
            let mut formatter = if self.use_colors {
                DefaultFormatter::new()
            } else {
                DefaultFormatter::without_colors()
            };
            
            formatter.show_timestamp = self.show_timestamp;
            formatter.show_target = self.show_target;
            formatter.show_location = self.show_location;
            formatter.show_thread = self.show_thread;
            formatter.level_width = self.level_width;
            formatter.timestamp_brackets = self.timestamp_brackets;
            formatter.level_brackets = self.level_brackets;
            formatter.target_brackets = self.target_brackets;
            formatter.time_format = self.config.time_format;
            
            Box::new(formatter)
        });

        // 创建控制台writer并应用中间件
        let console_writer = if self.use_colors {
            ConsoleWriter::with_formatter(console_formatter)
        } else {
            // 在无颜色模式下也使用配置的formatter
            ConsoleWriter::with_formatter(console_formatter)
                .with_color_support(false)
        };

        let mut console_writer: Box<dyn Writer> = Box::new(console_writer);

        // 应用中间件
        if !self.middlewares.is_empty() || self.metrics.is_some() {
            let mut middleware_writer = MiddlewareWriter::new(console_writer);
            
            // 注意：由于trait object不能克隆，这里需要转移所有权
            // 在实际应用中，可以考虑使用Arc<dyn LogMiddleware>来解决这个问题
            let middlewares = std::mem::take(&mut self.middlewares);
            for middleware in middlewares.into_iter() {
                middleware_writer = middleware_writer.with_middleware(middleware);
            }

            if let Some(ref metrics) = self.metrics {
                middleware_writer = middleware_writer.with_metrics(metrics.clone());
            }
            
            console_writer = Box::new(middleware_writer);
        }

        if self.config.async_logging {
            self.config.writers.push(Box::new(AsyncWriter::new(console_writer)));
        } else {
            self.config.writers.push(console_writer);
        }

        // 添加文件writer
        if let Some(ref path) = self.file_path {
            let file_formatter = self.file_formatter.unwrap_or_else(|| {
                let mut formatter = DefaultFormatter::without_colors();
                formatter.show_timestamp = self.show_timestamp;
                formatter.show_target = self.show_target;
                formatter.show_location = self.show_location;
                formatter.show_thread = self.show_thread;
                formatter.level_width = self.level_width;
                formatter.time_format = self.config.time_format;
                Box::new(formatter)
            });

            let file_writer = if let Some(ref policy) = self.rotation_policy {
                FileWriter::with_rotation(path, policy.clone())?
                    .max_backup_files(self.max_backup_files)
                    .append(self.file_append)
            } else {
                FileWriter::with_formatter(path, file_formatter)?
                    .append(self.file_append)
            };

            let file_writer: Box<dyn Writer> = Box::new(file_writer);
            
            if self.config.async_logging {
                self.config.writers.push(Box::new(AsyncWriter::new(file_writer)));
            } else {
                self.config.writers.push(file_writer);
            }
        }

        // 添加网络writers
        for (target, protocol) in self.network_writers {
            match protocol {
                NetworkProtocol::Tcp => {
                    if let Some(addr_str) = target.strip_prefix("tcp://") {
                        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                            let writer = NetworkWriter::tcp(addr)?;
                            if self.config.async_logging {
                                self.config.writers.push(Box::new(AsyncWriter::new(Box::new(writer))));
                            } else {
                                self.config.writers.push(Box::new(writer));
                            }
                        }
                    }
                },
                NetworkProtocol::Udp => {
                    if let Some(addr_str) = target.strip_prefix("udp://") {
                        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                            let writer = NetworkWriter::udp(addr)?;
                            if self.config.async_logging {
                                self.config.writers.push(Box::new(AsyncWriter::new(Box::new(writer))));
                            } else {
                                self.config.writers.push(Box::new(writer));
                            }
                        }
                    }
                },
                NetworkProtocol::Syslog { facility, .. } => {
                    if let Some(addr_str) = target.strip_prefix("syslog://") {
                        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                            let writer = SyslogWriter::new(addr, facility)?;
                            if self.config.async_logging {
                                self.config.writers.push(Box::new(AsyncWriter::new(Box::new(writer))));
                            } else {
                                self.config.writers.push(Box::new(writer));
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        // 添加自定义writers
        for writer in self.custom_writers {
            if self.config.async_logging {
                self.config.writers.push(Box::new(AsyncWriter::new(writer)));
            } else {
                self.config.writers.push(writer);
            }
        }

        Ok(Logger::from_config_with_metrics(self.config, self.metrics))
    }

    /// 初始化全局Logger
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let logger = self.build()?;
        init_with_logger(logger)
    }
}

/// 日志记录器 - 核心
pub struct Logger {
    config: LogConfig,
    metrics: Option<Arc<LoggerMetrics>>,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
            metrics: None,
        }
    }

    /// 从配置创建日志记录器
    pub fn from_config(config: LogConfig) -> Self {
        Self { 
            config,
            metrics: None,
        }
    }

    /// 从配置和指标创建日志记录器
    pub fn from_config_with_metrics(config: LogConfig, metrics: Option<Arc<LoggerMetrics>>) -> Self {
        Self { 
            config,
            metrics,
        }
    }

    /// 获取构建器
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// 记录日志核心函数
    pub fn log(
        &self,
        level: Level,
        target: &str,
        message: &str,
        file: Option<&str>,
        line: Option<u32>,
        module_path: Option<&str>,
    ) {
        if !self.config.level_filter.should_log(&level) {
            return;
        }

        let mut record = LogRecord::new(level, target, message);
        
        if let (Some(file), Some(line), Some(module)) = (file, line, module_path) {
            record = record.with_location(file, line, module);
        }

        for writer in &self.config.writers {
            writer.write(&record);
        }
    }

    /// 刷新所有writers
    pub fn flush(&self) {
        for writer in &self.config.writers {
            let _ = writer.flush();
        }
    }

    /// 检查级别是否启用
    pub fn is_enabled(&self, level: &Level) -> bool {
        self.config.level_filter.should_log(level)
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> Option<Arc<LoggerMetrics>> {
        self.metrics.clone()
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> Option<LoggerStats> {
        self.metrics.as_ref().map(|m| m.get_stats())
    }
}

/// 全局日志记录器
static GLOBAL_LOGGER: RwLock<Option<Logger>> = RwLock::new(None);

/// 初始化全局日志记录器
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::builder()
        .level(Level::info())
        .console()
        .build()?;
    
    let mut global_logger = GLOBAL_LOGGER.write().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

/// 使用配置初始化
pub fn init_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::from_config(config);
    let mut global_logger = GLOBAL_LOGGER.write().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

/// 使用Logger实例初始化
pub fn init_with_logger(logger: Logger) -> Result<(), Box<dyn std::error::Error>> {
    let mut global_logger = GLOBAL_LOGGER.write().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

/// 记录日志的内部函数
pub fn log(
    level: Level,
    target: &str,
    message: &str,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) {
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    if let Some(ref logger) = *logger_guard {
        logger.log(level, target, message, file, line, module_path);
    }
}

/// 记录日志但不输出到控制台的内部函数
pub fn log_without_console(
    level: Level,
    target: &str,
    message: &str,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) {
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    if let Some(ref logger) = *logger_guard {
        if !logger.config.level_filter.should_log(&level) {
            return;
        }

        let mut record = LogRecord::new(level, target, message);
        
        if let (Some(file), Some(line), Some(module)) = (file, line, module_path) {
            record = record.with_location(file, line, module);
        }

        // 只写入非控制台的writers
        for writer in &logger.config.writers {
            // 检查writer是否是ConsoleWriter类型
            if writer.as_any().downcast_ref::<ConsoleWriter>().is_none() {
                writer.write(&record);
            }
        }
    }
}

/// 便捷的初始化函数
pub fn init_simple(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    Logger::builder()
        .level_str(level)
        .console()
        .init()
}

pub fn init_with_file(level: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    Logger::builder()
        .level_str(level)
        .console()
        .file(file_path)
        .init()
}

/// 全局级别注册API - 注册自定义级别到全局Logger
pub fn register_global_level(name: &str, priority: u8, color: &str) -> Result<(), String> {
    register_level(name, priority, color)
}

/// 全局级别注册API - 注销自定义级别
pub fn unregister_global_level(name: &str) -> Result<(), String> {
    unregister_level(name)
}

/// 获取全局Logger是否已初始化
pub fn is_global_logger_initialized() -> bool {
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    logger_guard.is_some()
}

/// 设置全局日志等级
pub fn set_global_level<L: Into<Level>>(level: L) -> Result<(), String> {
    let mut logger_guard = GLOBAL_LOGGER.write().unwrap();
    if let Some(ref mut logger) = *logger_guard {
        logger.config.level_filter.set_min_level_with_level(level.into());
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 使用字符串设置全局日志等级
pub fn set_global_level_str(level: &str) -> Result<(), String> {
    let log_level = level.parse::<Level>().unwrap_or_else(|_| Level::Custom {
        name: level.to_string(),
        priority: 30,
        color: "".to_string(),
    });
    set_global_level(log_level)
}

/// 获取当前全局日志等级
pub fn get_global_min_level() -> Option<u8> {
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    logger_guard.as_ref().map(|logger| logger.config.level_filter.get_min_level())
}

/// 禁用全局级别
pub fn disable_global_level(level_name: &str) -> Result<(), String> {
    let mut logger_guard = GLOBAL_LOGGER.write().unwrap();
    if let Some(ref mut logger) = *logger_guard {
        logger.config.level_filter.disable_level(level_name);
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 启用全局级别
pub fn enable_global_level(level_name: &str) -> Result<(), String> {
    let mut logger_guard = GLOBAL_LOGGER.write().unwrap();
    if let Some(ref mut logger) = *logger_guard {
        logger.config.level_filter.enable_level(level_name);
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 批量禁用全局级别
pub fn disable_global_levels(level_names: &[&str]) -> Result<(), String> {
    let mut logger_guard = GLOBAL_LOGGER.write().unwrap();
    if let Some(ref mut logger) = *logger_guard {
        logger.config.level_filter.disable_levels(level_names);
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 批量启用全局级别
pub fn enable_global_levels(level_names: &[&str]) -> Result<(), String> {
    let mut logger_guard = GLOBAL_LOGGER.write().unwrap();
    if let Some(ref mut logger) = *logger_guard {
        logger.config.level_filter.enable_levels(level_names);
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 获取全局注册的级别
pub fn get_global_level(name: &str) -> Option<Level> {
    let level_name = name.to_uppercase();
    
    // 首先检查预定义级别
    match level_name.as_str() {
        "TRACE" => Some(Level::Predefined(PredefinedLevel::Trace)),
        "DEBUG" => Some(Level::Predefined(PredefinedLevel::Debug)),
        "NOTICE" => Some(Level::Predefined(PredefinedLevel::Notice)),
        "INFO" => Some(Level::Predefined(PredefinedLevel::Info)),
        "WARN" => Some(Level::Predefined(PredefinedLevel::Warn)),
        "ERROR" => Some(Level::Predefined(PredefinedLevel::Error)),
        "FATAL" => Some(Level::Predefined(PredefinedLevel::Fatal)),
        "RECORD" => Some(Level::Predefined(PredefinedLevel::Record)),
        _ => {
            // 检查全局注册表中的自定义级别
            let registry = GLOBAL_LEVEL_REGISTRY.read().unwrap();
            registry.get(&level_name).cloned()
        }
    }
}

/// 获取全局Logger的引用（如果存在）
pub fn with_global_logger<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Logger) -> R,
{
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    logger_guard.as_ref().map(f)
}

/// 使用字符串级别记录日志（支持自定义级别）
pub fn log_str(
    level: &str,
    target: &str,
    message: &str,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) -> Result<(), String> {
    let parsed_level = level.parse::<Level>().map_err(|_| format!("Invalid level: {level}"))?;
    let logger_guard = GLOBAL_LOGGER.read().unwrap();
    if let Some(ref logger) = *logger_guard {
        logger.log(parsed_level, target, message, file, line, module_path);
        Ok(())
    } else {
        Err("Global logger not initialized".to_string())
    }
}

/// 日志宏 - 兼容旧版本
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
macro_rules! notice {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::notice(), $target, $($arg)*);
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

#[macro_export]
macro_rules! fatal {
    ($target:expr, $($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::fatal(), $target, $($arg)*);
    };
}

#[macro_export]
macro_rules! record {
    ($($arg:tt)*) => {
        $crate::log!($crate::lycrex::logger::Level::record(), "lycrex", $($arg)*);
    };
}

#[macro_export]
macro_rules! record_without_console {
    ($($arg:tt)*) => {
        $crate::lycrex::logger::log_without_console(
            $crate::lycrex::logger::Level::record(),
            "lycrex",
            &format!($($arg)*),
            Some(file!()),
            Some(line!()),
            Some(module_path!())
        );
    };
}

// 默认宏
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
macro_rules! fatal_default {
    ($($arg:tt)*) => {
        $crate::fatal!("lycrex", $($arg)*);
    };
}

/// 使用字符串级别记录日志的宏
#[macro_export]
macro_rules! log_str {
    ($level:expr, $target:expr, $($arg:tt)*) => {
        let _ = $crate::lycrex::logger::log_str(
            $level,
            $target,
            &format!($($arg)*),
            Some(file!()),
            Some(line!()),
            Some(module_path!())
        );
    };
}

/// 使用字符串级别记录日志到默认target的宏
#[macro_export]
macro_rules! log_str_default {
    ($level:expr, $($arg:tt)*) => {
        $crate::log_str!($level, "lycrex", $($arg)*);
    };
}

/// 便捷的自定义级别日志宏
#[macro_export]
macro_rules! custom_log {
    ($level_name:expr, $target:expr, $($arg:tt)*) => {
        $crate::log_str!($level_name, $target, $($arg)*);
    };
}

/// 便捷的自定义级别日志宏（默认target）
#[macro_export]
macro_rules! custom_log_default {
    ($level_name:expr, $($arg:tt)*) => {
        $crate::log_str_default!($level_name, $($arg)*);
    };
}
