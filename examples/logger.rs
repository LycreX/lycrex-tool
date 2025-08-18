use lycrex_tool::lycrex::logger::{
    Logger, LogConfig, Level, LevelFilter, ConsoleWriter, FileWriter,
    register_global_level, get_global_level
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = LogConfig::default();
    config.level_filter = LevelFilter::new(Level::trace());
    let console_writer = ConsoleWriter::new();
    let file_writer = FileWriter::new("app.log".to_string());
    config.writers.push(Box::new(console_writer));
    config.writers.push(Box::new(file_writer?));
    let mut logger = Logger::from_config(config);
    
    logger.log(Level::trace(), "example", "This is a Trace message", None, None, None);
    logger.log(Level::debug(), "example", "This is a Debug message", None, None, None);
    logger.log(Level::info(), "example", "This is an Info message", None, None, None);
    logger.log(Level::notice(), "example", "This is a Notice message", None, None, None);
    logger.log(Level::warn(), "example", "This is a Warn message", None, None, None);
    logger.log(Level::error(), "example", "This is an Error message", None, None, None);
    logger.log(Level::fatal(), "example", "This is a Fatal message", None, None, None);

    let database_level = Level::custom("DATABASE", 15, "\x1b[35m");
    let security_level = Level::custom("SECURITY", 20, "\x1b[91m");
    let performance_level = Level::custom("PERF", 5, "\x1b[93m");
    
    logger.log(database_level.clone(), "db", "TEST DATABASE", None, None, None);
    logger.log(security_level.clone(), "auth", "TEST SECURITY", None, None, None);
    logger.log(performance_level.clone(), "app", "TEST PERF", None, None, None);

    logger.disable_level("ERROR");
    logger.log(Level::error(), "test", "This should be filtered", None, None, None);
    
    logger.enable_level("PERF");
    logger.log(performance_level.clone(), "app", "This should be displayed", None, None, None);
    
    logger.enable_level("ERROR");
    logger.log(Level::error(), "test", "ERROR level re-enabled", None, None, None);

    logger.disable_levels(&["WARN", "DATABASE", "SECURITY"]);
    
    logger.log(Level::warn(), "test", "This should be filtered", None, None, None);
    logger.log(database_level.clone(), "db", "This should be filtered", None, None, None);
    logger.log(Level::info(), "test", "This should be displayed", None, None, None);
    
    logger.enable_levels(&["WARN", "DATABASE"]);
    logger.log(Level::warn(), "test", "This should be displayed", None, None, None);
    logger.log(database_level, "db", "This should be displayed", None, None, None);
    
    register_global_level("API", 12, "\x1b[96m");
    register_global_level("CACHE", 8, "\x1b[94m");
    
    if let Some(api_level) = get_global_level("API") {
        logger.log(api_level, "api", "API call success", None, None, None);
    }
    
    if let Some(cache_level) = get_global_level("CACHE") {
        logger.log(cache_level, "cache", "Cache hit rate: 95%", None, None, None);
    }
    Ok(())
}
