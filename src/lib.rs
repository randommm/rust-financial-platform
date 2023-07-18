use futures_util::{SinkExt, StreamExt};

use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use std::str;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Deserialize, Debug)]
struct Data {
    p: f64,
    s: String,
    t: f64,
    v: f64,
}

#[derive(Deserialize, Debug)]
struct WSMessage {
    data: Vec<Data>,
    #[serde(rename = "type")]
    type_: String,
}

pub async fn run(
    connect_addr: String,
    database_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url.as_str())
        .await?;

    let mut url = url::Url::parse(&connect_addr)?;

    let (ws_stream, _) = connect_async(&url).await?;
    let (mut write, read) = ws_stream.split();

    url.set_query(None);
    println!("Connected to {}", url);

    write
        .send(Message::Text(
            r#"{"type":"subscribe","symbol":"BINANCE:BTCUSDT"}"#.to_owned(),
        ))
        .await
        .unwrap_or_default();
    write
        .send(Message::Text(
            r#"{"type":"subscribe","symbol":"IC MARKETS:1"}"#.to_owned(),
        ))
        .await
        .unwrap_or_default();

    read.for_each(|message| async {
        let data = if let Ok(message) = message {
            message.into_data()
        } else if let Err(e) = message {
            tokio::io::stdout()
                .write_all(format!("Error while decoding message: {}", e).as_bytes())
                .await
                .unwrap_or_default();
            return;
        } else {
            return;
        };
        match str::from_utf8(&data) {
            Ok(wsmes) => {
                if let Ok(wsmes) = serde_json::from_str::<WSMessage>(wsmes) {
                    if wsmes.type_ != "trade" {
                        tokio::io::stdout()
                            .write_all(format!("Invalid Json type {}", wsmes.type_).as_bytes())
                            .await
                            .unwrap_or_default();
                        return;
                    }
                    for data in wsmes.data.iter() {
                        if let Err(e) = sqlx::query(
                            "INSERT INTO trade (price, security, timestamp, volume)
                        VALUES (?, ?, ?, ?);",
                        )
                        .bind(data.p)
                        .bind(&data.s)
                        .bind(data.t)
                        .bind(data.v)
                        .execute(&db_pool)
                        .await
                        {
                            tokio::io::stdout()
                                .write_all(format!("Error while inserting data: {e}").as_bytes())
                                .await
                                .unwrap_or_default();
                        }
                    }
                } else {
                    tokio::io::stdout()
                        .write_all(format!("Invalid Json: {}", wsmes).as_bytes())
                        .await
                        .unwrap_or_default();
                }
            }
            Err(e) => {
                tokio::io::stdout()
                    .write_all(format!("Invalid UTF-8 sequence: {}", e).as_bytes())
                    .await
                    .unwrap_or_default();
            }
        };
    })
    .await;

    Ok(())
}
