use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;
use tracing_appender::rolling;
use std::io;

pub fn init_logger() {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_log = fmt::layer()
        .with_writer(io::stdout)
        .with_ansi(true)
        .with_target(true)
        .with_line_number(true)
        .pretty();

    let file_appender = rolling::daily("logs", "cmpler.log");
    let file_log = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_log)
        .with(file_log)
        .init();
}
