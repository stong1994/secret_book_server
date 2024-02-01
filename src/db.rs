
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    name :String,
    date :String,
    event_type :Types,
    content :String,
    from :String,
}

#[allow(clippy::enum_variant_names)]
pub enum Execute{
    FetchEvent(Option<String>),
    PushEvent(Event),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Types {
    CREATE,
    UPDATE,
    DELETE,
}

impl Types {
    fn as_str(&self)-> &'static str{
        match self {
            Types::CREATE=> "create",
            Types::UPDATE=> "update",
            Types::DELETE=> "delete",
        }
    } 
    pub fn parse(s: &str)->Result<Types, String> {
        match s {
            "create"=>Ok(Types::CREATE),
            "update"=>Ok(Types::UPDATE),
            "delete"=>Ok(Types::DELETE),
            other=> Err(format!("{} is not a valid types", other))
        }
    }
}

pub async fn fetch_event(pool: &SqlitePool, last_sync_date: Option<String>) -> Result<Vec<Event>, anyhow::Error> {
    //     let stmt= match last_sync_date {
    //         Some(date) => format!("
    // SELECT name, date, type as event_type, content, from_client
    //     FROM events
    //     WHERE date > '{}'
    //     ORDER BY date ASC LIMIT 10", date).as_str(),
    //         None => r#"
    // SELECT name, date, type as event_type, content, from_client
    //     FROM events
    //     ORDER BY date ASC LIMIT 10"#,
    //     };

        struct Row {
            name :String,
            date :String,
            event_type :String,
            content :String,
            from_client :String, 
        }

        let result :Vec<Row> = sqlx::query_as!(
            Row,
            r#"
    SELECT name, date, type as event_type, content, from_client
        FROM events
        ORDER BY date ASC LIMIT 10"#,
            // stmt,
        )
        .fetch_all(pool)
        .await?;
        // for (idx, row) in result.iter().enumerate(){
            // println!("{}", row.get("name"))
        // }
        let events = result
        .into_iter()
        .filter_map(|row: Row| {
            match Types::parse(row.event_type.as_str()){
                Ok(event_type) => Some(
            Event{
                name: row.name,
                date: row.date,
                event_type: event_type,
                content: row.content,
                from: row.from_client,
            } 
                ),
                Err(error)=>{
                    // todo log

                    None
                }
            }
        }).collect();
        
        Ok(events)

}
