use log::{error, info};
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

struct EventRow {
    id: String,
    name: String,
    date: String,
    event_type: String,
    data_id: String,
    data_type: String,
    content: String,
    from_client: String,
}

async fn fetch_event_by_date_stmt(
    pool: &SqlitePool,
    last_sync_date: Option<String>,
) -> Result<Vec<EventRow>, sqlx::Error> {
    let date = match last_sync_date {
        Some(date) => date,
        None => "".to_string(),
    };
    sqlx::query_as!(
        EventRow,
        r#"
    SELECT id, name, date, event_type, data_id, data_type, content, from_client
        FROM events
        WHERE date > ?
        ORDER BY date ASC LIMIT 10"#,
        date,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_event(
    pool: &SqlitePool,
    last_sync_date: Option<String>,
) -> Result<Vec<Event>, anyhow::Error> {
    let events = fetch_event_by_date_stmt(pool, last_sync_date)
        .await?
        .iter()
        .filter_map(|row: &EventRow| {
            match Types::parse(row.event_type.as_str()) {
                Ok(event_type) => Some(Event {
                    id: row.id.clone(),
                    name: row.name.clone(),
                    date: row.date.clone(),
                    event_type: event_type,
                    data_id: row.data_id.clone(),
                    data_type: row.data_type.clone(),
                    content: row.content.clone(),
                    from: row.from_client.clone(),
                }),
                Err(_error) => {
                    // todo log

                    None
                }
            }
        })
        .collect();

    Ok(events)
}

pub async fn push_event(pool: &SqlitePool, event: Event) -> Result<(), anyhow::Error> {
    let state = save_state(pool, event.clone()).await?;
    store_event(pool, event.clone(), state).await?;
    Ok(())
}

pub async fn store_event(pool: &SqlitePool, event: Event, state: FinalState) -> Result<(), anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let event_type = event.event_type.as_str();
    let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
        r#"INSERT INTO events(id, name, date, event_type, data_id, data_type, content, from_client)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
        id,
        state.name,
        state.date,
        event_type,
        state.id,
        state.data_type,
        state.content,
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
// pub async fn push_event(pool: &SqlitePool, name: String, date :String, event_type :String, content: String, from :String) -> Result<(),anyhow::Error> {
//     sqlx::query!(
//         r#"INSERT INTO events(name, date, type, content, from_client)
//         VALUES ($1,$2,$3,$4,$5)"#,
//         name,
//         date,
//         event_type,
//         content,
//         from,
//     )
//     .execute(pool).await?;
//     Ok(())
// }
//
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalState {
    id: String,
    name: String,
    date: String,
    data_type: String,
    content: String,
}

pub async fn save_state(pool: &SqlitePool, event: Event) -> Result<FinalState, anyhow::Error> {
    match event.event_type {
        Types::CREATE => {
            let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
                r#"INSERT INTO finalstates(id, name, date, data_type, content)
                VALUES ($1,$2,$3,$4,$5)"#,
                event.data_id,
                event.name,
                event.date,
                event.data_type,
                event.content,
            )
            .execute(pool)
            .await;
            if rst.is_err() {
                let e = rst.unwrap_err();
                error!("Error inserting event into the database: {}", e);
                return Err(e.into());
            }
            Ok(FinalState{
                id: event.data_id,
                name: event.name,
                date: event.date,
                data_type: event.data_type,
                content: event.content,
            })
        }
        Types::UPDATE => {
            let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
                r#"UPDATE finalstates SET name = $1, date = $2, data_type = $3, content = $4 WHERE id = $5"#,
                event.name,
                event.date,
                event.data_type,
                event.content,
                event.data_id,
            )
            .execute(pool)
            .await;
            if rst.is_err() {
                let e = rst.unwrap_err();
                error!("Error updating event in the database: {}", e);
                return Err(e.into());
            }
            Ok(FinalState{
                id: event.data_id,
                name: event.name,
                date: event.date,
                data_type: event.data_type,
                content: event.content,
            })
        }
        Types::DELETE => {
            let rst: Result<SqliteQueryResult, sqlx::Error> = sqlx::query!(
                r#"DELETE FROM finalstates WHERE id = $1"#,
                event.data_id,
            )
            .execute(pool)
            .await;
            if rst.is_err() {
                let e = rst.unwrap_err();
                error!("Error deleting event from the database: {}", e);
                return Err(e.into());
            }
            Ok(FinalState{
                id: event.id,
                name: event.name,
                date: event.date,
                data_type: event.data_type,
                content: event.content,
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
    SELECT id, name, date, data_type, content
        FROM finalstates
        WHERE id > ? and data_type = ?
        ORDER BY id ASC LIMIT 10"#,
        id, data_type
    )
    .fetch_all(pool)
    .await?;
        
    Ok(states)
}
