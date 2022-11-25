use std::fmt::Display;
use std::mem::MaybeUninit;
use std::sync::Once;

use rocket::serde::Deserialize;
use rocket::Config;

struct Logger {
    log_level: rocket::config::LogLevel,
}

#[derive(Deserialize)]
struct LoggerConfig {
    log: rocket::config::LogLevel,
}

impl Logger {
    fn init() -> &'static Self {
        // Create an uninitialized static
        static mut SINGLETON: MaybeUninit<Logger> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let config: LoggerConfig = Config::figment().extract::<LoggerConfig>().unwrap();
                // Make it
                let singleton = Self {
                    log_level: config.log,
                };

                // Store it to the static var, i.e. initialize it
                SINGLETON.write(singleton);
            });

            // Now we give out a shared reference to the data, which is safe to use
            // concurrently.
            SINGLETON.assume_init_ref()
        }
    }
}

impl Logger {
    fn get_log_level(&self) -> rocket::config::LogLevel {
        self.log_level
    }
}

#[allow(dead_code)]
pub fn warn<S>(message: S)
where
    S: Display,
{
    if matches!(
        Logger::init().get_log_level(),
        rocket::config::LogLevel::Normal
    ) {
        println!("[WARNING] {}", message);
    }
}

pub fn info<S>(message: S)
where
    S: Display,
{
    if matches!(
        Logger::init().get_log_level(),
        rocket::config::LogLevel::Normal
    ) {
        println!("[INFO] {}", message);
    }
}

pub fn debug<S>(message: S)
where
    S: Display,
{
    if matches!(
        Logger::init().get_log_level(),
        rocket::config::LogLevel::Debug
    ) {
        println!("[DEBUG] {}", message);
    }
}
