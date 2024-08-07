use actix_web::{web, HttpResponse, ResponseError};
use reqwest::StatusCode;
use sqlx::SqlitePool;

use crate::db::{self, Event};

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum SecretError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SecretError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SecretError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "got event", skip(db))]
pub async fn push_event(
    event: web::Json<Event>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, SecretError> {
    db::push_event(&db, event.0).await?;

    Ok(HttpResponse::Ok().json(()))
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().body("PONG")
}

#[derive(serde::Deserialize)]
pub struct FetchStatesParam {
    data_type: String,
    last_sync_id: Option<String>,
}

pub async fn fetch_states(
    parameters: web::Query<FetchStatesParam>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, SecretError> {
    let result = db::fetch_states(
        &db,
        parameters.data_type.to_owned(),
        parameters.last_sync_id.to_owned(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(serde::Deserialize)]
pub struct FetchStateParam {
    host: String,
}
pub async fn fetch_state(
    parameters: web::Query<FetchStateParam>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse, SecretError> {
    let result = db::fetch_state(&db, parameters.host.to_owned()).await?;

    Ok(HttpResponse::Ok().json(result))
}

// #[actix_web::main]
// async fn app() -> io::Result<()> {
//     let _worker_guard = log::init_subscriber("app".into(), "info".into());
//
//     let args: Vec<String> = std::env::args().collect();
//     let db_url = if args.len() > 1 {
//         args[1].clone()
//     } else {
//         std::env::var("SECRET_SERVER_DB_URL").unwrap_or("sqlite://db/secret.db".to_string())
//     };
//
//     info!("db path: {}", db_url);
//
//     let pool = SqlitePool::connect(&db_url).await.unwrap();
//     // log::info!("starting HTTP server at http://localhost:12345");
//
//     // start HTTP server
//     HttpServer::new(move || {
//         App::new()
//             // store db pool as Data object
//             .app_data(web::Data::new(pool.clone()))
//             .wrap(TracingLogger::default())
//             .wrap(
//                 Cors::default()
//                     .allow_any_origin()
//                     .allow_any_method()
//                     .allow_any_header()
//                     .supports_credentials(),
//             )
//             // .route("/fetch_events", web::get().to(fetch_events))
//             .route("/fetch_states", web::get().to(fetch_states))
//             .route("/fetch_state", web::get().to(fetch_state))
//             .route("/push", web::post().to(push_event))
//             .route("/ping", web::get().to(ping))
//     })
//     .bind(("localhost", 12346))?
//     .workers(2)
//     .run()
//     .await
// }
