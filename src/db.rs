use serde::{Deserialize, Deserializer, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    name: String,
    date: String,
    event_type: Types,
    content: String,
    from: String,
}

#[derive(Debug, Serialize)]
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
    name: String,
    date: String,
    event_type: String,
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
    SELECT name, date, type as event_type, content, from_client
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
                    name: row.name.clone(),
                    date: row.date.clone(),
                    event_type: event_type,
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
    let event_type = event.event_type.as_str();
    sqlx::query!(
        r#"INSERT INTO events(name, date, type, content, from_client)
        VALUES ($1,$2,$3,$4,$5)"#,
        event.name,
        event.date,
        event_type,
        event.content,
        event.from,
    )
    .execute(pool)
    .await?;
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
