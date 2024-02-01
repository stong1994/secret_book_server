use std::io;

use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use reqwest::StatusCode;
use sqlx::{sqlite::SqliteQueryResult, SqlitePool};

mod db;

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

#[derive(serde::Deserialize)]
pub struct FetchParam{
    last_sync_date: Option<String>
}

async fn fetch(parameters: web::Query<FetchParam>, db: web::Data<SqlitePool>) -> Result<HttpResponse, SecretError> {
    let result = db::fetch_event(&db, parameters.last_sync_date.to_owned()).await?;

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // connect to SQLite DB
    let db_url = String::from("sqlite://db/secret.db");
    // if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
    //     Sqlite::create_database(&db_url).await.unwrap();
    //     match cretea_schema(&db_url).await {
    //         Ok(_) => println!("Database created Sucessfully"),
    //         Err(e) => panic!("{}",e),
    //     }
    // }

    let pool = SqlitePool::connect(&db_url).await.unwrap();
    // log::info!("starting HTTP server at http://localhost:12345");

    // start HTTP server
    HttpServer::new(move || {
        App::new()
            // store db pool as Data object
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(web::resource("/fetch").route(web::get().to(fetch)))
            // .service(web::resource("/push").route(web::post().to(parallel_weather)))
    })
    .bind(("127.0.0.1", 12345))?
    .workers(2)
    .run()
    .await
}

async fn cretea_schema(db_url:&str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(&db_url).await?;
    let qry = 
    "CREATE TABLE IF NOT EXISTS events
        (
            name TEXT NOT NULL,
            date TEXT PRIMARY KEY     NOT NULL,,
            type TEXT NOT NUL,
            content TEXT NOT NULL,
            from TEXT NOT NULL,
        );";
    let result = sqlx::query(&qry).execute(&pool).await;   
    pool.close().await; 
    return result;
}