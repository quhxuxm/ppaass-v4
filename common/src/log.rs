use crate::{Error, ServerConfig};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init<C: ServerConfig>(config: &C) -> Result<WorkerGuard, Error> {
    let tracing_subscriber_registry = tracing_subscriber::registry();
    let (trace_file_appender, trace_appender_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::daily(config.log_directory(), config.log_name_prefix()),
    );
    tracing_subscriber_registry.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(config.max_log_level())
    })).with(tracing_subscriber::fmt::layer()
        .with_writer(trace_file_appender)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_ansi(false)
    ).init();
    Ok(trace_appender_guard)
}
