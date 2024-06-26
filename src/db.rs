use log::error;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{sqlite::SqliteQueryResult, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    id: String,
    name: String,
    date: String,
    event_type: Types,
    data_id: String,
    data_type: String,
    content: String,
    desc: String,
    from: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum Types {
    CREATE,
    UPDATE,
    DELETE,
}

impl Types {
    fn as_str(&self) -> &'static str {
        match self {
            Types::CREATE => "create",
            Types::UPDATE => "update",
            Types::DELETE => "delete",
        }
    }
    pub fn parse(s: &str) -> Result<Types, String> {
        match s {
            "create" => Ok(Types::CREATE),
            "update" => Ok(Types::UPDATE),
            "delete" => Ok(Types::DELETE),
            other => Err(format!("{} is not a valid types", other)),
        }
    }
}

impl<'de> Deserialize<'de> for Types {
    fn deserialize<D>(deserializer: D) -> Result<Types, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Types::parse(s).map_err(serde::de::Error::custom)
    }
}

pub async fn push_event(pool: &SqlitePool, event: Event) -> Result<(), anyhow::Error> {
    let state = save_state(pool, event.clone()).await?;
    store_event(pool, event.clone(), state).await?;
    Ok(())
}

pub async fn store_event(
    pool: &SqlitePool,
    event: Event,
    state: FinalState,
) -> Result<(), anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let event_type = event.event_type.as_str();
    let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
        r#"INSERT INTO events(id, name, date, event_type, data_id, data_type, content, desc, from_client)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        id,
        state.name,
        state.date,
        event_type,
        state.id,
        state.data_type,
        state.content,
        state.desc,
        event.from,
    )
    .execute(pool)
    .await;
    if rst.is_err() {
        let e = rst.unwrap_err();
        error!("Error inserting event into the database: {}", e);
        return Err(e.into());
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinalState {
    id: String,
    name: String,
    date: String,
    data_type: String,
    content: String,
    desc: String,
}

pub async fn save_state(pool: &SqlitePool, event: Event) -> Result<FinalState, anyhow::Error> {
    match event.event_type {
        Types::CREATE | Types::UPDATE => {
            let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
                r#"INSERT OR REPLACE INTO finalstates(id, name, date, data_type, content, desc)
                VALUES ($1,$2,$3,$4,$5,$6)"#,
                event.data_id,
                event.name,
                event.date,
                event.data_type,
                event.content,
                event.desc,
            )
            .execute(pool)
            .await;
            if rst.is_err() {
                let e = rst.unwrap_err();
                error!("Error inserting or updating event in the database: {}", e);
                return Err(e.into());
            }
            Ok(FinalState {
                id: event.data_id,
                name: event.name,
                date: event.date,
                data_type: event.data_type,
                content: event.content,
                desc: event.desc,
            })
        }
        Types::DELETE => {
            let rst: Result<SqliteQueryResult, sqlx::Error> =
                sqlx::query!(r#"DELETE FROM finalstates WHERE id = $1"#, event.data_id,)
                    .execute(pool)
                    .await;
            if rst.is_err() {
                let e = rst.unwrap_err();
                error!("Error deleting event from the database: {}", e);
                return Err(e.into());
            }
            Ok(FinalState {
                id: event.id,
                name: event.name,
                date: event.date,
                data_type: event.data_type,
                content: event.content,
                desc: event.desc,
            })
        }
    }
}

pub async fn fetch_states(
    pool: &SqlitePool,
    data_type: String,
    last_fetch_id: Option<String>,
) -> Result<Vec<FinalState>, anyhow::Error> {
    let id = match last_fetch_id {
        Some(date) => date,
        None => "".to_string(),
    };
    let states = sqlx::query_as!(
        FinalState,
        r#"
    SELECT id, name, date, data_type, content, desc
        FROM finalstates
        WHERE id > ? and data_type = ?
        ORDER BY id ASC LIMIT 10"#,
        id,
        data_type
    )
    .fetch_all(pool)
    .await?;

    Ok(states)
}
pub async fn fetch_state(
    pool: &SqlitePool,
    host: String,
) -> Result<Vec<FinalState>, anyhow::Error> {
    // Try to find states where name or desc contains the host
    let host_like = format!("%{}%", host);
    let mut states = sqlx::query_as!(
        FinalState,
        r#"
    SELECT id, name, date, data_type, content, desc
        FROM finalstates
        WHERE name LIKE ? OR desc LIKE ?"#,
        host_like,
        host_like,
    )
    .fetch_all(pool)
    .await?;

    // If no states found, try with parent domain
    if states.is_empty() {
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            let parent_domain = parts[1..].join(".");
            states = Box::pin(fetch_state(pool, parent_domain)).await?;
        }
    }

    Ok(states)
}
