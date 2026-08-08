use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize tracing subscriber with console output and rolling file appender
pub fn init_logging(log_dir: &Path) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(log_dir, "drill.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,drill=debug"));

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}
