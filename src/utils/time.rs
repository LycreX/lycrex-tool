use std::time::{SystemTime, UNIX_EPOCH, Instant};
use chrono::{DateTime, Local, Utc};

/// 时区信息
#[derive(Debug, Clone)]
pub struct TimezoneInfo {
    pub offset_hours: i32,
    pub offset_minutes: i32,
    pub name: String,
}

/// 时间格式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeFormat {
    /// Unix时间戳（秒）
    Unix,
    /// Unix时间戳（毫秒）
    UnixMillis,
    /// 系统时间（UTC偏移）
    SystemTime,
    /// 系统时间（本地时间）
    LocalTime,
    /// ISO 8601格式
    Iso8601,
    /// 程序运行时间
    Relative,
}

/// 时间信息结构体
#[derive(Debug, Clone)]
pub struct TimeInfo {
    pub unix: u64,
    pub unix_millis: u128,
    pub system_time: String,
    pub local_time: String,
    pub iso8601: String,
    pub relative: u128,
}

/// 获取系统时区信息
pub fn get_system_timezone() -> TimezoneInfo {
    let local: DateTime<Local> = Local::now();
    // let utc: DateTime<Utc> = Utc::now();
    
    // 计算时区偏移
    let offset = local.offset();
    let offset_seconds = offset.utc_minus_local();
    let offset_hours = offset_seconds / 3600;
    let offset_minutes = (offset_seconds % 3600) / 60;
    
    // 获取时区名称
    let timezone_name = format!("UTC{:+03}:{:02}", offset_hours, offset_minutes.abs());
    
    TimezoneInfo {
        offset_hours: offset_hours,
        offset_minutes: offset_minutes,
        name: timezone_name,
    }
}

/// 获取当前时间信息
pub fn get_current_time() -> TimeInfo {
    let now = SystemTime::now();
    let unix_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    
    TimeInfo {
        unix: unix_epoch.as_secs(),
        unix_millis: unix_epoch.as_millis(),
        system_time: format_system_time(),
        local_time: format_local_time(),
        iso8601: format_iso8601_time(),
        relative: get_program_uptime_millis(),
    }
}

/// 根据指定格式获取时间
pub fn get_time(format: TimeFormat) -> String {
    match format {
        TimeFormat::Unix => get_current_time().unix.to_string(),
        TimeFormat::UnixMillis => get_current_time().unix_millis.to_string(),
        TimeFormat::SystemTime => get_current_time().system_time,
        TimeFormat::LocalTime => get_current_time().local_time,
        TimeFormat::Iso8601 => get_current_time().iso8601,
        TimeFormat::Relative => get_current_time().relative.to_string(),
    }
}

/// 判断是否为闰年
pub fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// 获取指定年份的天数
pub fn get_days_in_year(year: i32) -> u32 {
    if is_leap_year(year) { 366 } else { 365 }
}

/// 获取指定月份的天数
pub fn get_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

/// 计算从Epoch到指定日期的天数
pub fn days_since_epoch(year: i32, month: u32, day: u32) -> i64 {
    let mut days = 0i64;
    
    for y in 1970..year {
        days += get_days_in_year(y) as i64;
    }
    
    for m in 1..month {
        days += get_days_in_month(year, m) as i64;
    }
    
    days + day as i64 - 1
}

/// 格式化系统时间（包含UTC偏移）
fn format_system_time() -> String {
    let local: DateTime<Local> = Local::now();
    let offset = local.offset();
    let offset_seconds = offset.utc_minus_local();
    let offset_hours = offset_seconds / 3600;
    let offset_minutes = (offset_seconds % 3600) / 60;
    
    let utc_offset = format!("{:+03}:{:02}", offset_hours, offset_minutes.abs());
    
    format!(
        "{} UTC{}",
        format!("{}", local.format("%Y-%m-%d %H:%M:%S%.6f")),
        utc_offset
    )
}

/// 格式化本地时间
fn format_local_time() -> String {
    let local: DateTime<Local> = Local::now();
    format!("{}", local.format("%Y-%m-%d %H:%M:%S"))
}

/// 格式化ISO 8601时间
fn format_iso8601_time() -> String {
    let utc: DateTime<Utc> = Utc::now();
    format!("{}", utc.format("%Y-%m-%dT%H:%M:%S%.6fZ"))
}

/// 程序启动时间（静态变量）
static mut PROGRAM_START: Option<Instant> = None;

/// 初始化程序启动时间
pub fn init_program_time() {
    unsafe {
        PROGRAM_START = Some(Instant::now());
    }
}

/// 获取程序运行时间（纳秒）
pub fn get_program_uptime_nanos() -> u128 {
    unsafe {
        match PROGRAM_START {
            Some(start_time) => start_time.elapsed().as_nanos(),
            None => {
                init_program_time();
                0
            }
        }
    }
}

/// 获取程序运行时间（微秒）
pub fn get_program_uptime_micros() -> u128 {
    unsafe {
        match PROGRAM_START {
            Some(start_time) => start_time.elapsed().as_micros(),
            None => {
                PROGRAM_START = Some(Instant::now());
                0
            }
        }
    }
}

/// 获取程序运行时间（毫秒）
pub fn get_program_uptime_millis() -> u128 {
    unsafe {
        match PROGRAM_START {
            Some(start_time) => start_time.elapsed().as_millis(),
            None => {
                PROGRAM_START = Some(Instant::now());
                0
            }
        }
    }
}

pub fn get_program_uptime_seconds() -> u128 {
    get_program_uptime_millis() / 1000
}

/// 时间工具结构体
pub struct TimeUtils;

impl TimeUtils {
    /// 获取当前时间戳（秒）
    pub fn unix_timestamp() -> u64 {
        get_current_time().unix
    }
    
    /// 获取当前时间戳（毫秒）
    pub fn unix_timestamp_millis() -> u128 {
        get_current_time().unix_millis
    }
    
    /// 获取系统时间（包含UTC偏移）
    pub fn system_time_string() -> String {
        get_current_time().system_time
    }

    /// 获取本地时间
    pub fn local_time_string() -> String {
        get_current_time().local_time
    }
    
    /// 获取ISO 8601格式时间
    pub fn iso8601_time_string() -> String {
        get_current_time().iso8601
    }

    /// 获取程序运行时间（自然时间）
    pub fn program_uptime_string() -> String {
        Self::format_natural_time(get_program_uptime_millis())
    }
    
    pub fn format_natural_time(millis: u128) -> String {
        if millis < 1_000 {
            format!("+{}ms", millis)
        } else {
            let total_seconds = millis / 1_000;
            let days = total_seconds / 86400;
            let hours = (total_seconds % 86400) / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            
            let mut result = String::new();
            
            if days > 0 {
                result.push_str(&format!("{}d", days));
            }
            if hours > 0 || days > 0 {
                result.push_str(&format!("{}h", hours));
            }
            if minutes > 0 || hours > 0 || days > 0 {
                result.push_str(&format!("{}m", minutes));
            }
            result.push_str(&format!("{}s", seconds));
            
            format!("+{}", result)
        }
    }
    
    /// 获取程序运行时间（秒）
    pub fn program_uptime_seconds() -> u128 {
        get_program_uptime_seconds()
    }

    pub fn program_uptime_millis() -> u128 {
        get_program_uptime_millis()
    }

    pub fn program_uptime_micros() -> u128 {
        get_program_uptime_micros()
    }

    pub fn program_uptime_nanos() -> u128 {
        get_program_uptime_nanos()
    }

    pub fn program_uptime(level: u8) -> u128 {
        match level {
            0 => get_program_uptime_seconds() as u128,
            1 => get_program_uptime_millis(),
            2 => get_program_uptime_micros(),
            3 => get_program_uptime_nanos(),
            _ => get_program_uptime_millis(),
        }
    }

    /// 获取完整时间信息
    pub fn full_time_info() -> TimeInfo {
        get_current_time()
    }
    
    /// 获取系统时区信息
    pub fn timezone_info() -> TimezoneInfo {
        get_system_timezone()
    }
    
    /// 获取本地时间
    pub fn local_time() -> DateTime<Local> {
        Local::now()
    }
    
    /// 获取UTC时间
    pub fn utc_time() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// 将UTC时间转换为本地时间
    pub fn utc_to_local(utc_time: DateTime<Utc>) -> DateTime<Local> {
        DateTime::from(utc_time)
    }
    
    /// 将本地时间转换为UTC时间
    pub fn local_to_utc(local_time: DateTime<Local>) -> DateTime<Utc> {
        DateTime::from(local_time)
    }
    
    /// 验证日期是否有效
    pub fn is_valid_date(year: i32, month: u32, day: u32) -> bool {
        if month < 1 || month > 12 {
            return false;
        }
        
        if day < 1 || day > get_days_in_month(year, month) {
            return false;
        }
        
        true
    }
    
    /// 计算两个日期之间的天数差
    pub fn days_between_dates(year1: i32, month1: u32, day1: u32, 
                             year2: i32, month2: u32, day2: u32) -> i64 {
        let days1 = days_since_epoch(year1, month1, day1);
        let days2 = days_since_epoch(year2, month2, day2);
        (days2 - days1).abs()
    }
    
    /// 获取指定日期的星期几（0=周日，1=周一，...，6=周六）
    pub fn get_weekday(year: i32, month: u32, day: u32) -> u32 {
        let days = days_since_epoch(year, month, day);
        // 1970年1月1日是周四，所以加4再模7
        ((days + 4) % 7) as u32
    }
    
    /// 获取星期几的名称
    pub fn get_weekday_name(weekday: u32) -> &'static str {
        match weekday {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Unknown",
        }
    }
    
    /// 获取月份的名称
    pub fn get_month_name(month: u32) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
    
    /// 格式化时间到指定格式
    pub fn format_time(format: &str) -> String {
        let local: DateTime<Local> = Local::now();
        local.format(format).to_string()
    }
    
    /// 解析时间字符串
    pub fn parse_time(time_str: &str, format: &str) -> Option<DateTime<Local>> {
        chrono::NaiveDateTime::parse_from_str(time_str, format)
            .ok()
            .and_then(|naive| naive.and_local_timezone(Local).single())
    }
}