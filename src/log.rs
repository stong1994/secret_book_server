use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::subscriber::set_global_default;
use tracing::{info, warn, Subscriber};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let skipped_fields = vec!["http.host", "http.flavor", "file"];
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout)
        .skip_fields(skipped_fields.into_iter())
        .expect("One of the specified fields cannot be skipped");
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscribe(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set subscriber");
}
