//! A simple, opiniated logger for command line tools
//!
//! `cli_logger` aims at a very simple thing: logging for CLI tools done right. It uses the
//! `log` crate and the `ansi_term` crate for colors. It provides very few configuration —
//! at this time, just the expected log level.
//!
//! ## Features
//!
//!  `cli_logger`:
//!
//! * logs everything to `stderr`. CLI tools are expected to be usable in a pipe. In that context,
//!   the messages addressed to the user must be written on `stderr` to have a chance to be read
//!   by the user, independently of the log level.
//!   The program outputs that are meant to be used with a pipe shouldn't go through the logging
//!   system, but instead be printed to `stdout`, for example with `println!`.
//! * shows the `Info` message as plain uncolored text. `Info` is expected to be the normal log
//!   level to display messages that are not highlighting a problem and that are not too verbose
//!   for a standard usage of the tool. Because it is intended for messages that are related
//!   to a normal situation, the messages of that level are not prefixed with the log level.
//! * prefix the messages with their colored log level for any level other than `Info`. The color
//!   depends on the log level, allowing to quickly locate a message at a specific log level
//! * displays the module path and line when configured at the `Trace` log level, for all the
//!   messages, even if they are not at the `Trace` log level. The `Trace` log level is used
//!   to help the developer understand where a message comes from, in addition to display a larger
//!   amount of messages.
//! * disable all colorization in case the `stderr` is not a tty, so the output is not polluted
//!   with unreadable characters when `stderr` is redirected to a file.
//!
//! ## Example with `Info` log level
//!
//! ```rust
//! #[macro_use] extern crate log;
//! extern crate cli_logger;
//!
//! fn main() {
//!      cli_logger::.init(log::Level::Info).unwrap();
//!
//!      error!("This is printed to stderr, with the 'error: ' prefix colored in red");
//!      warn!("This is printed to stderr, with the 'warn: ' prefix colored in yellow");
//!      info!("This is printed to stderr, without prefix or color");
//!      debug!("This is not printed");
//!      trace!("This is not printed");
//! }
//! ```
//!
//! ## Example with `Trace` log level
//!
//! ```rust
//! #[macro_use] extern crate log;
//! extern crate cli_logger;
//!
//! fn main() {
//!      cli_logger::.init(log::Level::Trace).unwrap();
//!
//!      error!("This is printed to stderr, with the 'path(line): error: ' prefix colored in red");
//!      warn!("This is printed to stderr, with the 'path(line): warn: ' prefix colored in yellow");
//!      info!(This is printed to stderr, with the 'path(line): info: ' prefix");
//!      debug!("This is printed to stderr, with the 'path(line): debug: ' prefix colored in blue");
//!      trace!(This is printed to stderr, with the 'path(line): trace: ' prefix colored in magenta");
//! }
//! ```
//!
//! ## Example with log level configured with a command line option
//!
//! TODO: write a small example that uses structopt
//!

use ansi_term::Color;
use log::SetLoggerError;

pub const MODULE_PATH_UNKNOWN: &str = "?";
pub const MODULE_LINE_UNKNOWN: &str = "?";

#[derive(Debug, Clone, PartialEq)]
pub struct Logger {
    level: log::Level,
}

impl Logger {
    /// Creates a new instance of the cli logger.
    ///
    /// The default level is Info.
    pub fn new() -> Logger {
        Logger {
            level: log::Level::Info,
        }
    }

    /// Explicitly sets the log level.
    pub fn level(mut self, l: log::Level) -> Self {
        self.level = l;
        self
    }

    /// Initializes the logger.
    ///
    /// This also consumes the logger. It cannot be further modified after initialization.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level.to_level_filter());
        log::set_boxed_logger(Box::new(self))
    }

    fn log_with_level(&self, record: &log::Record) {
        let level = record.level().to_string().to_lowercase();
        let header = match record.level() {
            log::Level::Info => "".to_string(),
            _ => format!("{}: ", level),
        };
        let header = paint(record.level(), &header);
        eprintln!("{}{}", header, record.args());
    }

    fn log_with_trace(&self, record: &log::Record) {
        let path = record.module_path().unwrap_or(MODULE_PATH_UNKNOWN);
        let line = if let Some(l) = record.line() {
            l.to_string()
        } else {
            MODULE_LINE_UNKNOWN.to_string()
        };
        let level = record.level().to_string().to_lowercase();
        let header = format!("{}({}): {}: ", path, line, level);
        let header = paint(record.level(), &header);
        eprintln!("{}{}", header, record.args());
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            match self.level {
                log::Level::Trace => self.log_with_trace(record),
                _ => self.log_with_level(record),
            }
        }
    }
    fn flush(&self) {
        // already done
    }
}

impl Default for Logger {
    fn default() -> Logger {
        Logger::new()
    }
}

/// Initializes the logger.
///
/// This also consumes the logger. It cannot be further modified after initialization.
///
/// # Example
///
/// ```rust
/// #[macro_use] extern crate log;
/// extern crate cli_logger;
///
/// fn main() {
///     cli_logger::.init(log::Level::Info).unwrap();
///
///     error!("This is printed to stderr, with the 'error: ' prefix");
///     warn!("This is printed to stderr, with the 'warn: ' prefix"");
///     info!("This is printed to stderr, without prefix");
///     debug!("This is not printed");
///     trace!("This is not printed");
/// }
/// ```
pub fn init(level: log::Level) -> Result<(), SetLoggerError> {
    Logger::new().level(level).init()
}

/// Colorize a string with the color associated with the log level
fn paint(level: log::Level, msg: &str) -> std::string::String {
    if atty::is(atty::Stream::Stderr) {
        match level {
            log::Level::Error => Color::Red.paint(msg).to_string(),
            log::Level::Warn => Color::Yellow.paint(msg).to_string(),
            log::Level::Info => msg.to_string(),
            log::Level::Debug => Color::Blue.paint(msg).to_string(),
            log::Level::Trace => Color::Purple.paint(msg).to_string(),
        }
    } else {
        msg.to_string()
    }
}
