use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpServer};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing_actix_web::TracingLogger;

use crate::{
    configurations::Settings,
    route::{fetch_state, fetch_states, ping, push_event},
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn build(configuration: Settings) -> Result<Self> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let lst = TcpListener::bind(address).context("Failed to bind port")?;
        let port = lst.local_addr().unwrap().port();
        let db_pool = SqlitePool::connect_lazy(&configuration.database.url)
            .context("Failed to create connection pool")?;
        let server = run(lst, db_pool)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await.context("Server stopped unexpectedly")
    }
}

pub struct ApplicationUrl(pub String);

pub fn run(lst: TcpListener, db_pool: SqlitePool) -> Result<Server> {
    let pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .route("/fetch_states", web::get().to(fetch_states))
            .route("/fetch_state", web::get().to(fetch_state))
            .route("/push", web::post().to(push_event))
            .route("/ping", web::get().to(ping))
            .app_data(pool.clone())
    })
    .listen(lst)?
    .run();
    Ok(server)
}
