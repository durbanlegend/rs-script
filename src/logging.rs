#![allow(clippy::uninlined_format_args)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

pub struct Logger {
    pub verbosity: Verbosity,
}

impl Logger {
    #[must_use]
    pub fn new(verbosity: Verbosity) -> Self {
        Logger { verbosity }
    }

    pub fn log(&self, verbosity: Verbosity, message: &str) {
        if verbosity as u8 <= self.verbosity as u8 {
            println!("{}", message);
        }
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }
}

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(Verbosity::Normal)); // Default to Normal
}

/// Sets the logging verbosity for the current execution.
/// # Panics
/// Will panic if the logger mutex cannot be unlocked..
pub fn set_global_verbosity(verbosity: Verbosity) {
    let mut logger = LOGGER.lock().unwrap();
    logger.set_verbosity(verbosity);
}

#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            let logger = $crate::logging::LOGGER.lock().unwrap();
            logger.log($verbosity, &format!($($arg)*));
        }
    };
}
