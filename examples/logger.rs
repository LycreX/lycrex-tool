use lycrex_tool::lycrex::logger::{
    disable_global_level, disable_global_levels, enable_global_level, 
    enable_global_levels, get_global_level, init_with_config, log, 
    register_global_level, ConsoleWriter, DefaultFormatter, FileWriter, 
    Level, LevelFilter, LogConfig
};
use lycrex_tool::utils::time::TimeFormat;
use lycrex_tool::{record, record_without_console};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut config = LogConfig::default();
    config.level_filter = LevelFilter::new(Level::trace());
    let file_writer = FileWriter::new("app.log".to_string());

    let console_formatter = DefaultFormatter {
        use_colors: true,
        show_timestamp: true,
        timestamp_color: "\x1b[2m",
        timestamp_with_square: false,
        level_in_right: true,
        level_width: 8 as usize,
        show_target: true,
        show_location: false,
        time_format: TimeFormat::Iso8601,
        uptime_level: -1,
    };
    let console_writer = ConsoleWriter::with_formatter(Box::new(console_formatter));
    config.writers.push(Box::new(console_writer));
    config.writers.push(Box::new(file_writer?));
    config.time_format = TimeFormat::Iso8601;
    let _ = init_with_config(config);

    log(Level::trace(), "example", "This is a Trace message", None, None, None);
    log(Level::debug(), "example", "This is a Debug message", None, None, None);
    log(Level::info(), "example", "This is an Info message", None, None, None);
    log(Level::notice(), "example", "This is a Notice message", None, None, None);
    log(Level::warn(), "example", "This is a Warn message", None, None, None);
    log(Level::error(), "example", "This is an Error message", None, None, None);
    log(Level::fatal(), "example", "This is a Fatal message", None, None, None);
    record!("This is a Record message");
    record_without_console!("This is a Record message without console");

    let database_level = Level::custom("DATABASE", 15, "\x1b[35m");
    let security_level = Level::custom("SECURITY", 20, "\x1b[91m");
    let performance_level = Level::custom("PERF", 5, "\x1b[93m");
    
    log(database_level.clone(), "db", "TEST DATABASE", None, None, None);
    log(security_level.clone(), "auth", "TEST SECURITY", None, None, None);
    log(performance_level.clone(), "app", "TEST PERF", None, None, None);

    disable_global_level("ERROR");
    log(Level::error(), "test", "This should be filtered", None, None, None);
    
    enable_global_level("PERF");
    log(performance_level.clone(), "app", "This should be displayed", None, None, None);
    
    enable_global_level("ERROR");
    log(Level::error(), "test", "ERROR level re-enabled", None, None, None);

    disable_global_levels(&["WARN", "DATABASE", "SECURITY"]);
    
    log(Level::warn(), "test", "This should be filtered", None, None, None);
    log(database_level.clone(), "db", "This should be filtered", None, None, None);
    log(Level::info(), "test", "This should be displayed", None, None, None);
    
    enable_global_levels(&["WARN", "DATABASE"]);
    log(Level::warn(), "test", "This should be displayed", None, None, None);
    log(database_level, "db", "This should be displayed", None, None, None);
    
    register_global_level("API", 12, "\x1b[96m");
    register_global_level("CACHE", 8, "\x1b[94m");
    
    if let Some(api_level) = get_global_level("API") {
        log(api_level, "api", "API call success", None, None, None);
    }
    
    if let Some(cache_level) = get_global_level("CACHE") {
        log(cache_level, "cache", "Cache hit rate: 95%", None, None, None);
    }
    Ok(())
}
