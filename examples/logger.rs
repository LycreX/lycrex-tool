use lycrex_tool::lycrex::logger::{
    Logger, Level, log, 
    register_global_level,
    disable_global_level, enable_global_level,
    disable_global_levels, enable_global_levels,
    get_global_level
};
use lycrex_tool::{record, record_without_console};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    Logger::builder()
        .level(Level::trace())
        .timestamp_brackets(false)
        .level_brackets(false)
        .show_target(true)
        .console()
        .file("app.log")
        .init()?;

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

    let _ = disable_global_level("ERROR");
    log(Level::error(), "test", "This should be filtered", None, None, None);
    
    let _ = enable_global_level("PERF");
    log(performance_level.clone(), "app", "This should be displayed", None, None, None);
    
    let _ = enable_global_level("ERROR");
    log(Level::error(), "test", "ERROR level re-enabled", None, None, None);

    let _ = disable_global_levels(&["WARN", "DATABASE", "SECURITY"]);
    
    log(Level::warn(), "test", "This should be filtered", None, None, None);
    log(database_level.clone(), "db", "This should be filtered", None, None, None);
    log(Level::info(), "test", "This should be displayed", None, None, None);
    
    let _ = enable_global_levels(&["WARN", "DATABASE"]);
    log(Level::warn(), "test", "This should be displayed", None, None, None);
    log(database_level, "db", "This should be displayed", None, None, None);
    
    let _ = register_global_level("API", 12, "\x1b[96m");
    let _ = register_global_level("CACHE", 8, "\x1b[94m");
    
    if let Some(api_level) = get_global_level("API") {
        log(api_level, "api", "API call success", None, None, None);
    }
    
    if let Some(cache_level) = get_global_level("CACHE") {
        log(cache_level, "cache", "Cache hit rate: 95%", None, None, None);
    }
    Ok(())
}
