#![forbid(unsafe_code)]
use std::env;

use either::{for_both, Either};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Re-export tracing for convenience.
pub use tracing;
use tracing_tree::HierarchicalLayer;

/// Re-export log macros for convenience.
pub use tracing::log::{debug, error, info, trace, warn};

pub fn setup_logger() {
    // This will print tracing events to standard output for humans to read
    let logger = tracing_subscriber::Registry::default()
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .with(
            HierarchicalLayer::new(3)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(false)
                .with_targets(true),
        );

    logger.init();
}

pub fn setup_logger_with_logfiles(logfile_prefix: &str) -> Option<WorkerGuard> {
    // This will print tracing events to standard output for humans to read
    let logger = tracing_subscriber::Registry::default()
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .with(
            HierarchicalLayer::new(3)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(false)
                .with_targets(true),
        );
    // When this variable goes out of scope (at the end of the function where this function is called), it will flush the log file writer
    let mut file_logger_guard = Option::None;

    // Masking the inner type using "dyn" keyword because return types are differents in the if / else
    let logger = if let Ok(directory) = env::var("LOG_DIRECTORY") {
        if !directory.is_empty() {
            let file_appender =
                tracing_appender::rolling::hourly(directory, format!("{}.log", logfile_prefix));
            let (log_writer, guard) = tracing_appender::non_blocking(file_appender);
            file_logger_guard = Some(guard);

            Either::Left(
                logger.with(
                    fmt::layer()
                        .with_writer(log_writer)
                        .with_ansi(false)
                        .compact(),
                ),
            )
        } else {
            Either::Right(logger)
        }
    } else {
        Either::Right(logger)
    };

    for_both!(logger, logger => logger.init());
    file_logger_guard
}
