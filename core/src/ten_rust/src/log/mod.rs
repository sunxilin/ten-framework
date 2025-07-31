//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod bindings;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use tracing::{Event, Subscriber};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::field::Visit;
use tracing_subscriber::fmt::{format, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{
    fmt::{self as tracing_fmt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, Registry,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "u8")]
pub enum LogLevel {
    Invalid = 0,
    Verbose = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
    Fatal = 6,
    Mandatory = 7,
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LogLevel::Invalid,
            1 => LogLevel::Verbose,
            2 => LogLevel::Debug,
            3 => LogLevel::Info,
            4 => LogLevel::Warn,
            5 => LogLevel::Error,
            6 => LogLevel::Fatal,
            7 => LogLevel::Mandatory,
            _ => LogLevel::Invalid,
        }
    }
}

impl LogLevel {
    fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Verbose => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Fatal => tracing::Level::ERROR,
            LogLevel::Mandatory => tracing::Level::INFO,
            LogLevel::Invalid => tracing::Level::INFO,
        }
    }
}

fn level_to_char(level: &tracing::Level) -> char {
    match *level {
        tracing::Level::TRACE => 'V',
        tracing::Level::DEBUG => 'D',
        tracing::Level::INFO => 'I',
        tracing::Level::WARN => 'W',
        tracing::Level::ERROR => 'E',
    }
}

/// Custom formatter to match the C plain formatter output
struct PlainFormatter;

impl<S, N> FormatEvent<S, N> for PlainFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();

        // Time - using ISO 8601 format
        use chrono::Utc;
        let now = Utc::now();
        write!(writer, "{}", now.to_rfc3339())?;

        // Extract fields from the event first
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        // PID(TID) - use values from C side
        let pid = visitor.pid.unwrap_or(0);
        let tid = visitor.tid.unwrap_or(0);
        write!(writer, " {pid}({tid}) ")?;

        // Level
        let level_char = level_to_char(metadata.level());
        write!(writer, "{level_char}")?;

        // Format function@file:line using extracted fields
        if let (Some(func_name), Some(file_name), Some(line_no)) = (
            visitor.func_name.as_ref(),
            visitor.file_name.as_ref(),
            visitor.line_no,
        ) {
            write!(writer, " {func_name}@{file_name}:{line_no}")?;
        } else {
            // Fallback to tracing's built-in metadata
            let target = metadata.target();
            if let Some(file) = metadata.file() {
                let line = metadata.line().unwrap_or(0);
                let filename = std::path::Path::new(file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file);

                if !target.is_empty() && target != "ten_log" {
                    write!(writer, " {target}@{filename}:{line}")?;
                } else {
                    write!(writer, " @{filename}:{line}")?;
                }
            } else if !target.is_empty() && target != "ten_log" {
                write!(writer, " {target}")?;
            }
        }

        // Message - only output the message part, skip the custom fields
        write!(writer, " ")?;
        if !visitor.message.is_empty() {
            write!(writer, "{}", visitor.message)?;
        } else {
            // Fallback to default field formatting
            ctx.field_format().format_fields(writer.by_ref(), event)?;
        }

        writeln!(writer)?;
        Ok(())
    }
}

#[derive(Default)]
struct FieldVisitor {
    pid: Option<i64>,
    tid: Option<i64>,
    func_name: Option<String>,
    file_name: Option<String>,
    line_no: Option<u32>,
    message: String,
}

impl Visit for FieldVisitor {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn fmt::Debug,
    ) {
        match field.name() {
            "pid" => {
                if let Ok(pid) = format!("{value:?}").parse::<i64>() {
                    self.pid = Some(pid);
                }
            }
            "tid" => {
                if let Ok(tid) = format!("{value:?}").parse::<i64>() {
                    self.tid = Some(tid);
                }
            }
            "func_name" => {
                self.func_name =
                    Some(format!("{value:?}").trim_matches('"').to_string());
            }
            "file_name" => {
                self.file_name =
                    Some(format!("{value:?}").trim_matches('"').to_string());
            }
            "line_no" => {
                if let Ok(line) = format!("{value:?}").parse::<u32>() {
                    self.line_no = Some(line);
                }
            }
            "message" => {
                if !self.message.is_empty() {
                    self.message.push(' ');
                }
                self.message.push_str(format!("{value:?}").trim_matches('"'));
            }
            _ => {
                // This might be the actual log message
                if field.name() == "message" || self.message.is_empty() {
                    if !self.message.is_empty() {
                        self.message.push(' ');
                    }
                    self.message
                        .push_str(format!("{value:?}").trim_matches('"'));
                }
            }
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "func_name" => {
                self.func_name = Some(value.to_string());
            }
            "file_name" => {
                self.file_name = Some(value.to_string());
            }
            "message" => {
                if !self.message.is_empty() {
                    self.message.push(' ');
                }
                self.message.push_str(value);
            }
            _ => {
                // This might be the actual log message
                if self.message.is_empty() {
                    self.message.push_str(value);
                }
            }
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "line_no" {
            self.line_no = Some(value as u32);
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        match field.name() {
            "pid" => {
                self.pid = Some(value);
            }
            "tid" => {
                self.tid = Some(value);
            }
            _ => {}
        }
    }
}

// Advanced log level enum that serializes to/from strings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AdvancedLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedLogMatcher {
    pub level: AdvancedLogLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FormatterType {
    Plain,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedLogFormatter {
    #[serde(rename = "type")]
    pub formatter_type: FormatterType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colored: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsoleEmitterConfig {
    pub stream: StreamType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEmitterConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "config")]
#[serde(rename_all = "lowercase")]
pub enum AdvancedLogEmitter {
    Console(ConsoleEmitterConfig),
    File(FileEmitterConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedLogHandler {
    pub matchers: Vec<AdvancedLogMatcher>,
    pub formatter: AdvancedLogFormatter,
    pub emitter: AdvancedLogEmitter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedLogConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<AdvancedLogLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub handlers: Option<Vec<AdvancedLogHandler>>,
}

/// Configure logging system using tracing library based on AdvancedLogConfig
///
/// # Features
/// - Support for multiple log handlers
/// - Filter logs by level and category
/// - Support for plain and JSON format output
/// - Support for console (stdout/stderr) and file output
/// - Support for colored output control
///
/// # Example
/// ```rust
/// use ten_rust::log::{
///     AdvancedLogConfig, AdvancedLogLevel, AdvancedLogHandler, AdvancedLogMatcher,
///     AdvancedLogFormatter, FormatterType, AdvancedLogEmitter, ConsoleEmitterConfig,
///     StreamType, ten_configure_log
/// };
///
/// let config = AdvancedLogConfig {
///     level: Some(AdvancedLogLevel::Info),
///     handlers: Some(vec![
///         AdvancedLogHandler {
///             matchers: vec![
///                 AdvancedLogMatcher {
///                     level: AdvancedLogLevel::Info,
///                     category: None, // Match all categories
///                 }
///             ],
///             formatter: AdvancedLogFormatter {
///                 formatter_type: FormatterType::Plain,
///                 colored: Some(true),
///             },
///             emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
///                 stream: StreamType::Stdout,
///             }),
///         }
///     ]),
/// };
///
/// ten_configure_log(&config);
///
/// // Now you can use tracing macros to log
/// tracing::info!("This is an info log");
/// tracing::warn!("This is a warning log");
/// ```
///
/// # Notes
/// - This function sets the global tracing subscriber and should only be called
///   once
/// - For file output, it's recommended to keep a reference to the guard
///   throughout the application lifecycle
/// - If no handlers are configured, default console output configuration will
///   be used
pub fn ten_configure_log(config: &AdvancedLogConfig) {
    // Create base registry
    let registry = Registry::default();

    // If no handlers are configured, use default configuration
    let handlers = match &config.handlers {
        Some(handlers) if !handlers.is_empty() => handlers,
        _ => {
            // Default configuration: output to stdout, use plain format
            let default_config = AdvancedLogConfig {
                level: config.level.clone(),
                handlers: Some(vec![AdvancedLogHandler {
                    matchers: vec![AdvancedLogMatcher {
                        level: config
                            .level
                            .clone()
                            .unwrap_or(AdvancedLogLevel::Info),
                        category: None,
                    }],
                    formatter: AdvancedLogFormatter {
                        formatter_type: FormatterType::Plain,
                        colored: Some(true),
                    },
                    emitter: AdvancedLogEmitter::Console(
                        ConsoleEmitterConfig { stream: StreamType::Stdout },
                    ),
                }]),
            };
            return ten_configure_log(&default_config);
        }
    };

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();

    // Create corresponding layer for each handler
    for handler in handlers {
        // Create filter
        let mut filter_directive = String::new();

        // Build filter rules based on matchers
        for (i, matcher) in handler.matchers.iter().enumerate() {
            if i > 0 {
                filter_directive.push(',');
            }

            let level_str = match matcher.level {
                AdvancedLogLevel::Trace => "trace",
                AdvancedLogLevel::Debug => "debug",
                AdvancedLogLevel::Info => "info",
                AdvancedLogLevel::Warn => "warn",
                AdvancedLogLevel::Error => "error",
            };

            if let Some(category) = &matcher.category {
                filter_directive.push_str(&format!("{category}={level_str}"));
            } else {
                filter_directive.push_str(level_str);
            }
        }

        let filter =
            EnvFilter::try_new(&filter_directive).unwrap_or_else(|_| {
                EnvFilter::new("info") // Default fallback to info level
            });

        // Create corresponding layer based on emitter type
        match &handler.emitter {
            AdvancedLogEmitter::Console(console_config) => {
                let layer: Box<dyn Layer<Registry> + Send + Sync> = match (
                    &console_config.stream,
                    &handler.formatter.formatter_type,
                ) {
                    (StreamType::Stdout, FormatterType::Plain) => {
                        tracing_fmt::Layer::new()
                            .event_format(PlainFormatter)
                            .with_writer(io::stdout)
                            .with_ansi(
                                handler.formatter.colored.unwrap_or(false),
                            )
                            .with_filter(filter)
                            .boxed()
                    }
                    (StreamType::Stderr, FormatterType::Plain) => {
                        tracing_fmt::Layer::new()
                            .event_format(PlainFormatter)
                            .with_writer(io::stderr)
                            .with_ansi(
                                handler.formatter.colored.unwrap_or(false),
                            )
                            // implementation
                            .with_filter(filter)
                            .boxed()
                    }
                    (StreamType::Stdout, FormatterType::Json) => {
                        tracing_fmt::Layer::new()
                            .json()
                            .with_ansi(
                                handler.formatter.colored.unwrap_or(false),
                            )
                            .with_writer(io::stdout)
                            .with_target(true)
                            .with_filter(filter)
                            .boxed()
                    }
                    (StreamType::Stderr, FormatterType::Json) => {
                        tracing_fmt::Layer::new()
                            .json()
                            .with_ansi(
                                handler.formatter.colored.unwrap_or(false),
                            )
                            .with_writer(io::stderr)
                            .with_target(true)
                            .with_filter(filter)
                            .boxed()
                    }
                };

                layers.push(layer);
            }
            AdvancedLogEmitter::File(file_config) => {
                // Create file appender for file logging
                let file_appender = rolling::never(".", &file_config.path);
                let (non_blocking, _guard) = non_blocking(file_appender);

                let layer = match handler.formatter.formatter_type {
                    FormatterType::Plain => {
                        tracing_fmt::Layer::new()
                            .event_format(PlainFormatter)
                            .with_writer(non_blocking)
                            .with_ansi(false) // File output doesn't need colors
                            .with_filter(filter)
                            .boxed()
                    }
                    FormatterType::Json => tracing_fmt::Layer::new()
                        .json()
                        .with_writer(non_blocking)
                        .with_target(true)
                        .with_filter(filter)
                        .boxed(),
                };

                layers.push(layer);

                // Note: _guard is dropped here, but in actual applications it
                // should be saved to ensure non_blocking writer
                // works properly
                std::mem::forget(_guard);
            }
        }
    }

    // Combine all layers and initialize global subscriber
    let subscriber = registry.with(layers);

    // Set global default subscriber
    if let Err(e) = subscriber.try_init() {
        eprintln!("Failed to set global default subscriber: {e}");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ten_log(
    _config: &AdvancedLogConfig,
    pid: i64,
    tid: i64,
    level: LogLevel,
    func_name: &str,
    file_name: &str,
    line_no: u32,
    msg: &str,
) {
    let tracing_level = level.to_tracing_level();

    // Extract just the filename from the full path
    let filename = std::path::Path::new(file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_name);

    match tracing_level {
        tracing::Level::TRACE => {
            tracing::trace!(
                pid = pid,
                tid = tid,
                func_name = func_name,
                file_name = filename,
                line_no = line_no,
                "{}",
                msg
            )
        }
        tracing::Level::DEBUG => {
            tracing::debug!(
                pid = pid,
                tid = tid,
                func_name = func_name,
                file_name = filename,
                line_no = line_no,
                "{}",
                msg
            )
        }
        tracing::Level::INFO => {
            tracing::info!(
                pid = pid,
                tid = tid,
                func_name = func_name,
                file_name = filename,
                line_no = line_no,
                "{}",
                msg
            )
        }
        tracing::Level::WARN => {
            tracing::warn!(
                pid = pid,
                tid = tid,
                func_name = func_name,
                file_name = filename,
                line_no = line_no,
                "{}",
                msg
            )
        }
        tracing::Level::ERROR => {
            tracing::error!(
                pid = pid,
                tid = tid,
                func_name = func_name,
                file_name = filename,
                line_no = line_no,
                "{}",
                msg
            )
        }
    }
}
