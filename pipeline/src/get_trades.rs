use crate::SECURITIES;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::PgPool;
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

pub async fn get_trades(
    connect_addr: String,
    db_pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        let mut url = url::Url::parse(&connect_addr)?;

        println!("Attempting to connect");
        let conn = connect_async(&url).await;
        let Ok((ws_stream, _)) = conn else {
            println!("Connection failed, will attempt to connect again");
            continue;
        };
        let (mut write, read) = ws_stream.split();

        url.set_query(None);
        println!("Connected to {}", url);

        for security in SECURITIES {
            write
                .send(Message::Text(format!(
                    r#"{{"type":"subscribe","symbol":"{}"}}"#,
                    security
                )))
                .await
                .unwrap_or_default();
        }

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
                            if wsmes.type_ != "ping" {
                                tokio::io::stdout()
                                    .write_all(
                                        format!("Invalid Json type {}", wsmes.type_).as_bytes(),
                                    )
                                    .await
                                    .unwrap_or_default();
                            }
                            return;
                        }
                        for data in wsmes.data.iter() {
                            if data.v == 0.0 {
                                continue;
                            }
                            while let Err(e) = sqlx::query(
                                "INSERT INTO trades_raw (price, security, timestamp, volume)
                            VALUES ($1, $2, $3, $4);",
                            )
                            .bind(data.p)
                            .bind(&data.s)
                            .bind(data.t)
                            .bind(data.v)
                            .execute(db_pool)
                            .await
                            {
                                tokio::io::stdout()
                                    .write_all(
                                        format!(
                                            "Error while inserting data into trades_raw table: {e}"
                                        )
                                        .as_bytes(),
                                    )
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
    }
}
