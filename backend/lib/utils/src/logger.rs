use std::fmt::Display;
use std::sync::OnceLock;

use rocket::config::LogLevel;
use rocket::serde::Deserialize;
use rocket::Config;

struct Logger {
    log_level: LogLevel,
}

#[derive(Deserialize)]
struct LoggerConfig {
    log: LogLevel,
}

impl Logger {
    fn init() -> &'static Self {
        static SINGLETON: OnceLock<Logger> = OnceLock::new();

        SINGLETON.get_or_init(|| {
            let config: LoggerConfig = Config::figment().extract::<LoggerConfig>().unwrap();

            Self {
                log_level: config.log,
            }
        })
    }
}

impl Logger {
    fn get_log_level(&self) -> LogLevel {
        self.log_level
    }
}

/// `info` and `warn` messages are shown at the `normal` level and at the more
/// verbose `debug` level.
fn shows_info(level: LogLevel) -> bool {
    matches!(level, LogLevel::Normal | LogLevel::Debug)
}

/// `debug` messages are shown only at the `debug` level.
fn shows_debug(level: LogLevel) -> bool {
    matches!(level, LogLevel::Debug)
}

#[allow(dead_code)]
pub fn warn<S>(message: S)
where
    S: Display,
{
    if shows_info(Logger::init().get_log_level()) {
        println!("[WARNING] {}", message);
    }
}

pub fn info<S>(message: S)
where
    S: Display,
{
    if shows_info(Logger::init().get_log_level()) {
        println!("[INFO] {}", message);
    }
}

pub fn debug<S>(message: S)
where
    S: Display,
{
    if shows_debug(Logger::init().get_log_level()) {
        println!("[DEBUG] {}", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::figment::{providers::Serialized, Figment};

    #[test]
    fn test_info_is_shown_at_normal_level() {
        assert!(shows_info(LogLevel::Normal));
    }

    #[test]
    fn test_info_is_shown_at_debug_level() {
        // Regression: Rocket.toml sets `log = "debug"` for the debug profile,
        // and info/warn messages were hidden in that profile.
        assert!(shows_info(LogLevel::Debug));
    }

    #[test]
    fn test_info_is_hidden_at_critical_and_off_levels() {
        assert!(!shows_info(LogLevel::Critical));
        assert!(!shows_info(LogLevel::Off));
    }

    #[test]
    fn test_debug_is_shown_only_at_debug_level() {
        assert!(shows_debug(LogLevel::Debug));
        assert!(!shows_debug(LogLevel::Normal));
        assert!(!shows_debug(LogLevel::Critical));
        assert!(!shows_debug(LogLevel::Off));
    }

    #[test]
    fn test_logger_config_reads_the_rocket_log_key() {
        let figment = Figment::from(Serialized::default("log", "critical"));
        let config: LoggerConfig = figment.extract().unwrap();

        assert_eq!(config.log, LogLevel::Critical);
    }

    #[test]
    fn test_logger_config_rejects_unknown_level() {
        let figment = Figment::from(Serialized::default("log", "verbose"));

        assert!(figment.extract::<LoggerConfig>().is_err());
    }

    #[test]
    fn test_logger_init_reads_level_from_rocket_figment() {
        // Rocket's default figment always has a `log` value, so init must not panic
        // and must agree with Rocket's own configuration.
        let expected = Config::figment().extract_inner::<LogLevel>("log").unwrap();

        assert_eq!(Logger::init().get_log_level(), expected);
    }

    #[test]
    fn test_logger_init_returns_the_same_instance() {
        assert!(std::ptr::eq(Logger::init(), Logger::init()));
    }

    #[test]
    fn test_log_functions_do_not_panic() {
        info("info message");
        warn(String::from("warn message"));
        debug(42);
    }
}
