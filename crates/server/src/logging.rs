//! This module setup the logger of `tracing`.
//!
//! In the initialization stage, the log writer will be guarded by their file
//! descriptor, the lifetime of the file descriptor is the same as the log
//! writer. You should keep the file descriptor until application exit.

use thiserror::Error;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{fmt::Layer, prelude::*, EnvFilter};

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file logger init error")]
    FileLoggerInitError(#[from] rolling::InitError),
}

/// Initialize the logger.
pub async fn initialize() -> Result<WorkerGuard, LoggerError> {
    let (non_blocking_console, _console_guard) = non_blocking(std::io::stdout());

    let console_log_layer = Layer::new()
        .with_writer(non_blocking_console)
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(console_log_layer)
        .init();

    Ok(_console_guard)
}
