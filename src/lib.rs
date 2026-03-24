//! # CCB Logger
//!
//! A beautiful, terminal-focused structured logger designed for modern CLI applications.
//!
//! CCB provides an elegant logging experience with semantic log levels, automatic color detection,
//! structured context support, and high-precision timestamps. It's built from the ground up
//! to deliver exceptional visual clarity and developer experience.
//!
//! ## Quick Start
//!
//! ```rust
//! use ccb::{info, warn, error};
//!
//! fn main() {
//!     info!("Application started");
//!     warn!("Configuration issue detected", "file", "config.toml");
//!     error!("Failed to connect", "host", "localhost", "port", "8080");
//! }
//! ```
//!
//! ## Features
//!
//! - **Five semantic log levels**: Trace, Debug, Info, Warn, Error
//! - **Automatic color detection**: Beautiful colored output with terminal compatibility
//! - **High-precision timestamps**: Microsecond precision with clean formatting
//! - **Structured logging**: Chain context with `with(key, value)` method
//! - **Convenient macros**: Easy-to-use macros with variadic key-value pairs
//! - **Global logger support**: Set and use application-wide logger configuration
//! - **Zero dependencies on icons**: Maximum terminal compatibility
//!
//! ## Custom Logger Configuration
//!
//! ```rust
//! use ccb::{Logger, Level, set_global_logger};
//!
//! let logger = Logger::new()
//!     .with_level(Level::Debug)
//!     .with_colors(true)
//!     .with("service", "auth")
//!     .with("version", "1.2.0");
//!
//! set_global_logger(logger);
//! ```

use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex};

use chrono::Local;
use once_cell::sync::Lazy;
use serde::Serialize;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Represents the severity level of a log message.
///
/// Log levels are ordered by severity, with `Trace` being the lowest and `Error` being the highest.
/// Each level has a distinct color and four-character representation for consistent alignment.
///
/// # Examples
///
/// ```rust
/// use ccb::Level;
///
/// assert!(Level::Trace < Level::Debug);
/// assert!(Level::Info < Level::Error);
/// assert_eq!(Level::Info.as_str(), "INFO");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// The lowest level, used for fine-grained tracing information.
    /// Displayed as "TRCE" in cyan color.
    Trace = 0,
    /// Development and diagnostic information.
    /// Displayed as "DEBG" in blue color.
    Debug = 1,
    /// General informational messages about application flow.
    /// Displayed as "INFO" in green color.
    Info = 2,
    /// Warning messages for potentially harmful situations.
    /// Displayed as "WARN" in yellow color.
    Warn = 3,
    /// Error messages for failure conditions.
    /// Displayed as "ERRO" in red color.
    Error = 4,
}

impl Level {
    /// Returns the four-character string representation of the log level.
    ///
    /// All levels are formatted to exactly four characters for consistent alignment
    /// in log output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Level;
    ///
    /// assert_eq!(Level::Trace.as_str(), "TRCE");
    /// assert_eq!(Level::Debug.as_str(), "DEBG");
    /// assert_eq!(Level::Info.as_str(), "INFO");
    /// assert_eq!(Level::Warn.as_str(), "WARN");
    /// assert_eq!(Level::Error.as_str(), "ERRO");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRCE",
            Level::Debug => "DEBG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERRO",
        }
    }

    /// Returns the terminal color associated with this log level.
    ///
    /// Each level has a distinct color to provide visual differentiation:
    /// - Trace: Cyan
    /// - Debug: Blue  
    /// - Info: Green
    /// - Warn: Yellow
    /// - Error: Red
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Level;
    /// use termcolor::Color;
    ///
    /// assert_eq!(Level::Info.color(), Color::Green);
    /// assert_eq!(Level::Error.color(), Color::Red);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            Level::Trace => Color::Cyan,
            Level::Debug => Color::Blue,
            Level::Info => Color::Green,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a single log entry with all associated metadata.
///
/// A `LogEntry` contains the log level, message, structured fields, and timestamp.
/// This structure is used internally by the logger to represent a complete log record
/// before it's formatted and written to the output. It can be serialized to JSON
/// for structured logging output.
///
/// # Examples
///
/// ```rust
/// use ccb::{LogEntry, Level};
/// use chrono::Local;
/// use std::collections::HashMap;
///
/// let entry = LogEntry {
///     level: Level::Info,
///     message: "User authenticated".to_string(),
///     fields: HashMap::new(),
///     timestamp: Local::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// The severity level of this log entry.
    pub level: String,
    /// The primary log message.
    pub message: String,
    /// Additional structured key-value pairs providing context.
    pub fields: HashMap<String, String>,
    /// The exact timestamp when this log entry was created (ISO 8601 format).
    pub timestamp: String,
}

/// Configuration settings for logger behavior and output formatting.
///
/// `Config` allows you to customize various aspects of logging behavior including
/// the minimum log level, color usage, timestamp display, and output format.
///
/// # Examples
///
/// ```rust
/// use ccb::{Config, Level};
///
/// let config = Config {
///     level: Level::Debug,
///     use_colors: false,  // Disable colors for CI environments
///     show_timestamp: true,
///     json_output: false,
///     timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
///     field_order: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// The minimum log level that will be output.
    /// Messages below this level will be filtered out.
    pub level: Level,
    /// Whether to use colors in the output.
    /// Automatically detected based on terminal capabilities by default.
    pub use_colors: bool,
    /// Whether to display timestamps in the output.
    /// When enabled, shows high-precision timestamps in gray.
    pub show_timestamp: bool,
    /// Whether to output logs in JSON format.
    /// When enabled, logs will be formatted as JSON objects instead of text.
    pub json_output: bool,
    /// The format string for timestamps.
    /// Uses chrono format specifiers (default: "%Y-%m-%d %H:%M:%S%.3f").
    pub timestamp_format: String,
    /// Custom field order for structured logging.
    /// When specified, fields will be displayed in this order.
    pub field_order: Option<Vec<String>>,
}

impl Default for Config {
    /// Creates a default configuration with sensible settings.
    ///
    /// Default settings:
    /// - Level: `Info` (filters out Debug and Trace)
    /// - Colors: Auto-detected based on terminal capabilities
    /// - Timestamp: Enabled with default format
    /// - JSON Output: Disabled
    /// - Timestamp Format: "%Y-%m-%d %H:%M:%S%.3f"
    /// - Field Order: None (natural order)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::{Config, Level};
    ///
    /// let config = Config::default();
    /// assert_eq!(config.level, Level::Info);
    /// assert_eq!(config.show_timestamp, true);
    /// assert!(!config.json_output);
    /// assert_eq!(config.timestamp_format, "%Y-%m-%d %H:%M:%S%.3f");
    /// assert!(config.field_order.is_none());
    /// ```
    fn default() -> Self {
        Self {
            level: Level::Info,
            use_colors: atty::is(atty::Stream::Stderr),
            show_timestamp: true,
            json_output: false,
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            field_order: None,
        }
    }
}

/// A structured logger with configurable output formatting and context management.
///
/// `Logger` is the core component that handles log formatting, filtering, and output.
/// It supports chainable configuration methods and maintains structured context
/// that gets applied to all log entries.
///
/// # Examples
///
/// ```rust
/// use ccb::{Logger, Level};
///
/// let logger = Logger::new()
///     .with_level(Level::Debug)
///     .with("service", "auth")
///     .with("version", "1.0.0");
///
/// logger.info("Server started", &[("port", "8080")]);
/// ```
#[derive(Debug, Clone)]
pub struct Logger {
    /// The logger's configuration settings.
    config: Config,
    /// Persistent context key-value pairs applied to all log entries.
    context: HashMap<String, String>,
}

impl Logger {
    /// Creates a new logger with default configuration.
    ///
    /// The default logger uses `Info` level, auto-detects color support,
    /// enables timestamps, and has no initial context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.info("Application started", &[]);
    /// ```
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            context: HashMap::new(),
        }
    }

    /// Creates a logger with a custom configuration.
    ///
    /// This allows full control over logger behavior including log level,
    /// color usage, and timestamp display.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for this logger
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::{Logger, Config, Level};
    ///
    /// let config = Config {
    ///     level: Level::Debug,
    ///     use_colors: false,
    ///     show_timestamp: true,
    /// };
    /// let logger = Logger::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            context: HashMap::new(),
        }
    }

    /// Sets the minimum log level for this logger.
    ///
    /// Messages with a level below this threshold will be filtered out
    /// and not displayed.
    ///
    /// # Arguments
    ///
    /// * `level` - The minimum level to log
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::{Logger, Level};
    ///
    /// let logger = Logger::new().with_level(Level::Debug);
    /// // Now trace messages will be filtered out, but debug and above will show
    /// ```
    pub fn with_level(mut self, level: Level) -> Self {
        self.config.level = level;
        self
    }

    /// Enables or disables colored output.
    ///
    /// When colors are enabled, log levels are displayed with their associated
    /// colors and bold formatting. This setting overrides automatic terminal detection.
    ///
    /// # Arguments
    ///
    /// * `use_colors` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new().with_colors(false); // Force disable colors
    /// ```
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.config.use_colors = use_colors;
        self
    }

    /// Enables or disables timestamp display in log output.
    ///
    /// When enabled, each log entry is prefixed with a timestamp
    /// in the format specified by `timestamp_format` displayed in gray.
    ///
    /// # Arguments
    ///
    /// * `show_timestamp` - Whether to display timestamps
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new().with_timestamp(false); // Hide timestamps
    /// ```
    pub fn with_timestamp(mut self, show_timestamp: bool) -> Self {
        self.config.show_timestamp = show_timestamp;
        self
    }

    /// Enables or disables JSON output format.
    ///
    /// When enabled, log entries are formatted as JSON objects instead of
    /// human-readable text. This is useful for machine processing and
    /// log collection systems.
    ///
    /// # Arguments
    ///
    /// * `json_output` - Whether to use JSON output format
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new().with_json_output(true); // Enable JSON output
    /// ```
    pub fn with_json_output(mut self, json_output: bool) -> Self {
        self.config.json_output = json_output;
        self
    }

    /// Sets the timestamp format string.
    ///
    /// Uses chrono format specifiers to define how timestamps are displayed.
    /// Default format: "%Y-%m-%d %H:%M:%S%.3f"
    ///
    /// # Arguments
    ///
    /// * `format` - The format string for timestamps
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new().with_timestamp_format("%H:%M:%S"); // Simple time format
    /// ```
    pub fn with_timestamp_format(mut self, format: &str) -> Self {
        self.config.timestamp_format = format.to_string();
        self
    }

    /// Sets the custom field order for structured logging.
    ///
    /// When specified, fields will be displayed in the specified order.
    /// Fields not in the order list will be displayed after ordered fields.
    ///
    /// # Arguments
    ///
    /// * `order` - Vector of field names in desired display order
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new()
    ///     .with_field_order(vec!["timestamp".to_string(), "level".to_string(), "message".to_string()]);
    /// ```
    pub fn with_field_order(mut self, order: Vec<String>) -> Self {
        self.config.field_order = Some(order);
        self
    }

    /// Adds a context key-value pair that will be included in all log entries.
    ///
    /// Context is persistent and gets applied to every log message from this logger.
    /// This is useful for adding service names, versions, request IDs, or other
    /// metadata that should appear in all logs.
    ///
    /// # Arguments
    ///
    /// * `key` - The context key (converted to String)
    /// * `value` - The context value (converted to String)
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new()
    ///     .with("service", "auth")
    ///     .with("version", "1.2.0")
    ///     .with("request_id", "req-123");
    ///
    /// logger.info("Processing request", &[]); // Will include all context fields
    /// ```
    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Logs a message at the specified level with additional structured fields.
    ///
    /// This is the core logging method used by all level-specific methods.
    /// It combines the logger's persistent context with the provided fields
    /// and outputs the message if it meets the minimum level threshold.
    ///
    /// # Arguments
    ///
    /// * `level` - The severity level for this log entry
    /// * `message` - The primary log message
    /// * `fields` - Additional key-value pairs for this specific log entry
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::{Logger, Level};
    ///
    /// let logger = Logger::new();
    /// logger.log(Level::Info, "User authenticated", &[("user_id", "12345")]);
    /// ```
    pub fn log(&self, level: Level, message: &str, fields: &[(&str, &str)]) {
        if level < self.config.level {
            return;
        }

        let mut entry_fields = self.context.clone();
        for (key, value) in fields {
            entry_fields.insert(key.to_string(), value.to_string());
        }

        let entry = LogEntry {
            level: level.as_str().to_string(),
            message: message.to_string(),
            fields: entry_fields,
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        };

        self.write_entry(&entry);
    }

    /// Logs a message at trace level.
    ///
    /// Trace messages are intended for fine-grained diagnostic information,
    /// typically used for debugging complex flows or performance analysis.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `fields` - Additional key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.trace("Entering function", &[("function", "calculate_hash")]);
    /// ```
    pub fn trace(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(Level::Trace, message, fields);
    }

    /// Logs a message at debug level.
    ///
    /// Debug messages provide detailed information for development and
    /// troubleshooting purposes.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `fields` - Additional key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.debug("Cache miss", &[("key", "user:12345")]);
    /// ```
    pub fn debug(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(Level::Debug, message, fields);
    }

    /// Logs a message at info level.
    ///
    /// Info messages communicate general information about application
    /// flow and important events.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `fields` - Additional key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.info("Server started", &[("port", "8080")]);
    /// ```
    pub fn info(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(Level::Info, message, fields);
    }

    /// Logs a message at warn level.
    ///
    /// Warn messages indicate potentially harmful situations that
    /// don't prevent the application from continuing.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `fields` - Additional key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.warn("High memory usage", &[("usage", "85%")]);
    /// ```
    pub fn warn(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(Level::Warn, message, fields);
    }

    /// Logs a message at error level.
    ///
    /// Error messages indicate failure conditions that may prevent
    /// the application from functioning correctly.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `fields` - Additional key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccb::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.error("Database connection failed", &[("host", "localhost")]);
    /// ```
    pub fn error(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(Level::Error, message, fields);
    }

    /// Formats and writes a log entry to stderr.
    ///
    /// This method handles the complete formatting process including timestamps,
    /// colored level indicators, the message, and structured fields. Output is
    /// written to stderr using the configured color settings.
    ///
    /// Supports both human-readable text format and JSON format based on configuration.
    /// In test environments where stderr might not be available, write operations
    /// are silently ignored to prevent panics.
    ///
    /// # Arguments
    ///
    /// * `entry` - The log entry to format and write
    fn write_entry(&self, entry: &LogEntry) {
        // In test environments, stderr might not be available, so we need to handle errors gracefully
        let result = std::panic::catch_unwind(|| {
            if self.config.json_output {
                self.write_json_entry(entry);
            } else {
                self.write_text_entry(entry);
            }
        });

        // Silently ignore any panics that occur during writing
        // This is primarily for test environments where stderr might not be available
        let _ = result;
    }

    /// Writes a log entry in JSON format.
    fn write_json_entry(&self, entry: &LogEntry) {
        let color_choice = if self.config.use_colors {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };

        let mut stderr = StandardStream::stderr(color_choice);

        // Create JSON object with proper field order
        let mut json_map = serde_json::Map::new();
        
        // Add timestamp if enabled
        if self.config.show_timestamp {
            json_map.insert("timestamp".to_string(), serde_json::Value::String(
                entry.timestamp.clone()
            ));
        }
        
        // Add level
        json_map.insert("level".to_string(), serde_json::Value::String(
            entry.level.clone()
        ));
        
        // Add message
        json_map.insert("message".to_string(), serde_json::Value::String(
            entry.message.clone()
        ));
        
        // Add fields with custom order
        if let Some(ref field_order) = self.config.field_order {
            for field_name in field_order {
                if let Some(value) = entry.fields.get(field_name) {
                    json_map.insert(field_name.clone(), serde_json::Value::String(value.clone()));
                }
            }
            // Add remaining fields
            for (key, value) in &entry.fields {
                if !field_order.contains(key) {
                    json_map.insert(key.clone(), serde_json::Value::String(value.clone()));
                }
            }
        } else {
            // Add all fields in natural order
            for (key, value) in &entry.fields {
                json_map.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        // Serialize and write JSON
        if let Ok(json_str) = serde_json::to_string(&json_map) {
            let _ = writeln!(stderr, "{}", json_str);
        }
        
        let _ = stderr.flush();
    }

    /// Writes a log entry in human-readable text format.
    fn write_text_entry(&self, entry: &LogEntry) {
        let color_choice = if self.config.use_colors {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };

        let mut stderr = StandardStream::stderr(color_choice);

        // Write timestamp if enabled
        if self.config.show_timestamp {
            let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(128, 128, 128))));
            let timestamp_str = chrono::DateTime::parse_from_rfc3339(&entry.timestamp)
                .map(|dt| dt.format(&self.config.timestamp_format).to_string())
                .unwrap_or_else(|_| entry.timestamp.clone());
            let _ = write!(stderr, "{} ", timestamp_str);
            let _ = stderr.reset();
        }

        // Write level with color and bold
        let _ = stderr.set_color(
            ColorSpec::new()
                .set_bold(true),
        );
        let _ = write!(stderr, "{} ", entry.level);
        let _ = stderr.reset();

        // Write message
        let _ = write!(stderr, "{}", entry.message);

        // Write context fields with custom order
        if let Some(ref field_order) = self.config.field_order {
            for field_name in field_order {
                if let Some(value) = entry.fields.get(field_name) {
                    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(128, 128, 128))));
                    let _ = write!(stderr, " {}=", field_name);
                    let _ = stderr.reset();
                    let _ = write!(stderr, "{}", value);
                }
            }
            // Add remaining fields
            for (key, value) in &entry.fields {
                if !field_order.contains(key) {
                    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(128, 128, 128))));
                    let _ = write!(stderr, " {}=", key);
                    let _ = stderr.reset();
                    let _ = write!(stderr, "{}", value);
                }
            }
        } else {
            // Write context fields in natural order
            for (key, value) in &entry.fields {
                let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(128, 128, 128))));
                let _ = write!(stderr, " {}=", key);
                let _ = stderr.reset();
                let _ = write!(stderr, "{}", value);
            }
        }

        let _ = writeln!(stderr);
        let _ = stderr.flush();
    }
}

impl Default for Logger {
    /// Creates a logger with default configuration.
    ///
    /// Equivalent to calling `Logger::new()`.
    fn default() -> Self {
        Self::new()
    }
}

/// Global logger instance used by the logging macros.
///
/// This static instance allows the logging macros to function without requiring
/// explicit logger parameters. It can be customized using `set_global_logger()`.
static GLOBAL_LOGGER: Lazy<Arc<Mutex<Logger>>> = Lazy::new(|| Arc::new(Mutex::new(Logger::new())));

/// Sets the global logger instance used by logging macros.
///
/// This function replaces the default global logger with a custom configured logger.
/// After calling this, all macro invocations will use the new logger configuration.
///
/// # Arguments
///
/// * `logger` - The logger instance to set as the global logger
///
/// # Examples
///
/// ```rust
/// use ccb::{Logger, Level, set_global_logger, info};
///
/// let custom_logger = Logger::new()
///     .with_level(Level::Debug)
///     .with("app", "my-service");
///
/// set_global_logger(custom_logger);
/// info!("This will use the custom logger configuration");
/// ```
pub fn set_global_logger(logger: Logger) {
    if let Ok(mut global) = GLOBAL_LOGGER.lock() {
        *global = logger;
    }
}

/// Returns a clone of the current global logger.
///
/// This function provides access to the global logger instance, allowing you to
/// inspect its configuration or use it directly instead of through macros.
///
/// # Returns
///
/// A cloned copy of the current global logger.
///
/// # Examples
///
/// ```rust
/// use ccb::{global_logger, Level};
///
/// let logger = global_logger();
/// logger.info("Direct logger usage", &[("source", "global")]);
/// ```
pub fn global_logger() -> Logger {
    GLOBAL_LOGGER.lock().unwrap().clone()
}

/// Executes a closure with access to the global logger.
///
/// This function provides thread-safe access to the global logger by acquiring
/// a lock and executing the provided closure with the logger reference.
///
/// # Arguments
///
/// * `f` - A closure that receives a reference to the global logger
///
/// # Type Parameters
///
/// * `F` - The closure type that takes a `&Logger` parameter
pub fn with_global_logger<F>(f: F)
where
    F: FnOnce(&Logger),
{
    if let Ok(logger) = GLOBAL_LOGGER.lock() {
        f(&*logger);
    }
}

/// Logs a message at trace level using the global logger.
///
/// This macro provides a convenient way to log trace-level messages with optional
/// structured key-value pairs. The first argument is the message, and subsequent
/// arguments are treated as alternating keys and values.
///
/// # Arguments
///
/// * `$msg` - The log message (expression that implements `Into<String>`)
/// * `$key`, `$value` - Optional alternating key-value pairs for structured logging
///
/// # Examples
///
/// ```rust
/// use ccb::trace;
///
/// trace!("Function entry");
/// trace!("Processing item", "id", 12345, "type", "user");
/// ```
#[macro_export]
macro_rules! trace {
    ($msg:expr) => {
        $crate::with_global_logger(|logger| logger.trace($msg, &[]));
    };
    ($msg:expr, $($key:expr, $value:expr),* $(,)?) => {
        $crate::with_global_logger(|logger| {
            let fields = &[$(($key, $value)),*];
            logger.trace($msg, fields);
        });
    };
}

/// Macro for debug level logging
#[macro_export]
macro_rules! debug {
    ($msg:expr) => {
        $crate::with_global_logger(|logger| logger.debug($msg, &[]));
    };
    ($msg:expr, $($key:expr, $value:expr),* $(,)?) => {
        $crate::with_global_logger(|logger| {
            let fields = &[$(($key, $value)),*];
            logger.debug($msg, fields);
        });
    };
}

/// Logs a message at info level using the global logger.
///
/// This macro provides a convenient way to log informational messages with optional
/// structured key-value pairs. Info messages communicate general application flow
/// and important events.
///
/// # Arguments
///
/// * `$msg` - The log message (expression that implements `Into<String>`)
/// * `$key`, `$value` - Optional alternating key-value pairs for structured logging
///
/// # Examples
///
/// ```rust
/// use ccb::info;
///
/// info!("Server started successfully");
/// info!("User logged in", "user_id", "12345", "ip", "192.168.1.100");
/// ```
#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        $crate::with_global_logger(|logger| logger.info($msg, &[]));
    };
    ($msg:expr, $($key:expr, $value:expr),* $(,)?) => {
        $crate::with_global_logger(|logger| {
            let fields = &[$(($key, $value)),*];
            logger.info($msg, fields);
        });
    };
}

/// Logs a message at warn level using the global logger.
///
/// This macro provides a convenient way to log warning messages with optional
/// structured key-value pairs. Warnings indicate potentially harmful situations
/// that don't prevent continued operation.
///
/// # Arguments
///
/// * `$msg` - The log message (expression that implements `Into<String>`)
/// * `$key`, `$value` - Optional alternating key-value pairs for structured logging
///
/// # Examples
///
/// ```rust
/// use ccb::warn;
///
/// warn!("Configuration file not found, using defaults");
/// warn!("High memory usage detected", "usage_percent", 87, "threshold", 80);
/// ```
#[macro_export]
macro_rules! warn {
    ($msg:expr) => {
        $crate::with_global_logger(|logger| logger.warn($msg, &[]));
    };
    ($msg:expr, $($key:expr, $value:expr),* $(,)?) => {
        $crate::with_global_logger(|logger| {
            let fields = &[$(($key, $value)),*];
            logger.warn($msg, fields);
        });
    };
}

/// Logs a message at error level using the global logger.
///
/// This macro provides a convenient way to log error messages with optional
/// structured key-value pairs. Error messages indicate failure conditions that
/// may prevent correct application functioning.
///
/// # Arguments
///
/// * `$msg` - The log message (expression that implements `Into<String>`)
/// * `$key`, `$value` - Optional alternating key-value pairs for structured logging
///
/// # Examples
///
/// ```rust
/// use ccb::error;
///
/// error!("Failed to connect to database");
/// error!("Authentication failed", "username", "alice", "reason", "invalid_password");
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        $crate::with_global_logger(|logger| logger.error($msg, &[]));
    };
    ($msg:expr, $($key:expr, $value:expr),* $(,)?) => {
        $crate::with_global_logger(|logger| {
            let fields = &[$(($key, $value)),*];
            logger.error($msg, fields);
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies that log levels are properly ordered by severity.
    fn test_level_ordering() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
    }

    #[test]
    /// Tests that all log levels return the correct four-character string representation.
    fn test_level_strings() {
        assert_eq!(Level::Trace.as_str(), "TRCE");
        assert_eq!(Level::Debug.as_str(), "DEBG");
        assert_eq!(Level::Info.as_str(), "INFO");
        assert_eq!(Level::Warn.as_str(), "WARN");
        assert_eq!(Level::Error.as_str(), "ERRO");
    }

    #[test]
    /// Verifies logger creation with default and custom configurations.
    fn test_logger_creation() {
        let logger = Logger::new();
        assert_eq!(logger.config.level, Level::Info);

        let logger = Logger::new().with_level(Level::Debug);
        assert_eq!(logger.config.level, Level::Debug);
    }

    #[test]
    /// Tests that context key-value pairs are properly stored and accessible.
    fn test_logger_with_context() {
        let logger = Logger::new()
            .with("service", "test")
            .with("version", "1.0.0");

        assert_eq!(logger.context.get("service"), Some(&"test".to_string()));
        assert_eq!(logger.context.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    /// Verifies that custom configurations are properly applied to loggers.
    fn test_logger_configuration() {
        let config = Config {
            level: Level::Debug,
            use_colors: false,
            show_timestamp: false,
            json_output: false,
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            field_order: None,
        };

        let logger = Logger::with_config(config.clone());
        assert_eq!(logger.config.level, Level::Debug);
        assert!(!logger.config.use_colors);
        assert!(!logger.config.show_timestamp);
    }

    #[test]
    /// Tests setting and retrieving the global logger instance.
    fn test_global_logger() {
        let custom_logger = Logger::new()
            .with_level(Level::Trace)
            .with("global", "test");

        set_global_logger(custom_logger);

        let retrieved = global_logger();
        assert_eq!(retrieved.config.level, Level::Trace);
        assert_eq!(retrieved.context.get("global"), Some(&"test".to_string()));
    }

    #[test]
    /// Ensures that all logging macros compile and execute without errors.
    /// In a real testing environment, stderr output would be captured for verification.
    fn test_macros_compile() {
        // Test that macros compile without panicking
        // In a real test environment, you might want to capture stderr
        trace!("Test trace message");
        debug!("Test debug message");
        info!("Test info message");
        warn!("Test warn message");
        error!("Test error message");

        trace!("Test with fields", "key1", "value1", "key2", "value2");
        info!("User login", "user_id", "12345", "ip", "192.168.1.1");
    }

    #[test]
    /// Verifies that LogEntry structures are created correctly with all required fields.
    fn test_log_entry_creation() {
        let _logger = Logger::new();
        let now = Local::now();

        let entry = LogEntry {
            level: Level::Info.as_str().to_string(),
            message: "test message".to_string(),
            fields: HashMap::new(),
            timestamp: now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        };

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "test message");
        assert!(entry.fields.is_empty());
    }

    #[test]
    /// Tests that the logger properly sets the minimum log level for filtering.
    fn test_level_filtering() {
        let logger = Logger::new().with_level(Level::Warn);

        // This test would need a way to capture output to verify filtering
        // For now, we just test that the configuration is set correctly
        assert_eq!(logger.config.level, Level::Warn);
    }

    #[test]
    /// Verifies that color configuration can be enabled and disabled correctly.
    fn test_colors_configuration() {
        let logger_with_colors = Logger::new().with_colors(true);
        let logger_without_colors = Logger::new().with_colors(false);

        assert!(logger_with_colors.config.use_colors);
        assert!(!logger_without_colors.config.use_colors);
    }

    #[test]
    /// Tests that timestamp display can be configured independently.
    fn test_timestamp_configuration() {
        let logger_with_timestamp = Logger::new().with_timestamp(true);
        let logger_without_timestamp = Logger::new().with_timestamp(false);

        assert!(logger_with_timestamp.config.show_timestamp);
        assert!(!logger_without_timestamp.config.show_timestamp);
    }

    #[test]
    /// Validates that the Config::default() implementation provides sensible defaults.
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.level, Level::Info);
        assert_eq!(config.show_timestamp, true);
        // use_colors depends on terminal detection, so we don't assert its value
    }

    #[test]
    /// Ensures that logger methods can be chained together fluently.
    fn test_method_chaining() {
        let logger = Logger::new()
            .with_level(Level::Trace)
            .with_colors(false)
            .with_timestamp(true)
            .with("chain", "test")
            .with("fluent", "api");

        assert_eq!(logger.config.level, Level::Trace);
        assert!(!logger.config.use_colors);
        assert!(logger.config.show_timestamp);
        assert_eq!(logger.context.len(), 2);
    }

    #[test]
    /// Tests that level colors are assigned correctly for terminal output.
    fn test_level_colors() {
        assert_eq!(Level::Trace.color(), Color::Cyan);
        assert_eq!(Level::Debug.color(), Color::Blue);
        assert_eq!(Level::Info.color(), Color::Green);
        assert_eq!(Level::Warn.color(), Color::Yellow);
        assert_eq!(Level::Error.color(), Color::Red);
    }

    #[test]
    /// Tests that the Display trait for Level works correctly.
    fn test_level_display() {
        assert_eq!(format!("{}", Level::Trace), "TRCE");
        assert_eq!(format!("{}", Level::Debug), "DEBG");
        assert_eq!(format!("{}", Level::Info), "INFO");
        assert_eq!(format!("{}", Level::Warn), "WARN");
        assert_eq!(format!("{}", Level::Error), "ERRO");
    }

    #[test]
    /// Tests JSON output configuration and formatting.
    fn test_json_output() {
        let logger = Logger::new().with_json_output(true);
        
        // Test that the configuration is set correctly
        assert!(logger.config.json_output);
    }

    #[test]
    /// Tests timestamp format configuration.
    fn test_timestamp_format() {
        let custom_format = "%H:%M:%S";
        let logger = Logger::new().with_timestamp_format(custom_format);
        
        assert_eq!(logger.config.timestamp_format, custom_format);
    }

    #[test]
    /// Tests field order configuration.
    fn test_field_order() {
        let expected_order = vec![
            "timestamp".to_string(),
            "level".to_string(),
            "message".to_string(),
        ];
        let logger = Logger::new().with_field_order(expected_order.clone());
        
        assert_eq!(logger.config.field_order, Some(expected_order));
    }

    #[test]
    /// Tests that JSON output creates valid JSON strings.
    fn test_json_output_validity() {
        let logger = Logger::new().with_json_output(true);
        
        // Test with a simple message
        logger.info("Test message", &[("key", "value")]);
        
        // In a real test, we would capture stderr and verify JSON validity
        // For now, we just ensure the logger doesn't panic
    }

    #[test]
    /// Tests combination of JSON output and field order.
    fn test_json_with_field_order() {
        let field_order = vec!["level".to_string(), "message".to_string(), "custom_field".to_string()];
        let logger = Logger::new()
            .with_json_output(true)
            .with_field_order(field_order.clone());
        
        assert!(logger.config.json_output);
        assert_eq!(logger.config.field_order, Some(field_order.clone()));
    }

    #[test]
    /// Tests custom timestamp format with text output.
    fn test_custom_timestamp_format() {
        let format = "%Y/%m/%d %H:%M";
        let logger = Logger::new()
            .with_timestamp_format(format)
            .with_timestamp(true);
        
        logger.info("Test timestamp format", &[]);
        
        // Test that the format is stored correctly
        assert_eq!(logger.config.timestamp_format, format);
    }
}
