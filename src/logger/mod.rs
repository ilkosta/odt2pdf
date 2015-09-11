extern crate time;
extern crate log;
extern crate fern;

use super::config as config;

pub fn init() {
    let conf = config::get_config();
    let logger_level = match conf.logger.level.as_ref() {
      "error" => log::LogLevelFilter::Error,
      "warn" => log::LogLevelFilter::Warn,
      "info"  => log::LogLevelFilter::Info,
      "debug"  => log::LogLevelFilter::Debug,
      "trace"  => log::LogLevelFilter::Trace,
      _ => log::LogLevelFilter::Warn
    };
    
    
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            format!("[{}][{}] {}", self::time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout(), fern::OutputConfig::file(&conf.logger.output)],
        level: logger_level
    };
    
    
    

    if let Err(e) = fern::init_global_logger(logger_config, logger_level) {
      panic!("Failed to initialize global logger: {}", e);
    }
}
