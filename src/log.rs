use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::{WorkerGuard};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_subscriber(name: String, env_filter: String) -> WorkerGuard {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let skipped_fields = vec!["http.host", "http.flavor", "file"];

    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let formatting_layer_file = BunyanFormattingLayer::new(name, file_writer)
        .skip_fields(skipped_fields.into_iter())
        .expect("One of the specified fields cannot be skipped");
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer_file);
    set_global_default(subscriber).expect("Failed to set subscriber");
    guard
}
