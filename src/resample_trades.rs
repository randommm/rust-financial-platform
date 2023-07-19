use crate::SECURITIES;

use futures::{future::join_all, TryStreamExt};
use sqlx::Row;
use tokio::time::{interval, sleep, Duration};

pub async fn resample_trades(db_pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Configurations
    // Maximum update frequency in milliseconds
    let mut interval = interval(Duration::from_millis(10));

    // Resample frequency in milliseconds
    let frequency = 100;

    // Past recheck leap in milliseconds
    // This is the amount of time in milliseconds that the resampler will
    // recheck for wrong data in the past
    // this is usefull because sometimes, the websocket connection will
    // give really out of order
    let past_recheck_leap = std::cmp::max(2 * frequency, 2000);

    loop {
        let mut tasks = Vec::new();
        for security in SECURITIES {
            tasks.push(async move {
                let max_timestamp: Result<(i64,), _> =
                    sqlx::query_as(r#"SELECT MAX(timestamp) FROM trade WHERE security = ?;"#)
                        .bind(security)
                        .fetch_one(db_pool)
                        .await;

                let Ok(max_timestamp) = max_timestamp else {return};

                let max_timestamp = max_timestamp.0.div_euclid(frequency) * frequency;

                //println!("max {}", max_timestamp);

                let min_timestamp: (i64,) = sqlx::query_as(
                    r#"SELECT MAX(timestamp) FROM resampled_trade WHERE security = ?;"#,
                )
                .bind(security)
                .fetch_one(db_pool)
                .await
                .unwrap_or_default();

                let min_timestamp = min_timestamp.0 - past_recheck_leap;

                //println!("min {}", min_timestamp);

                let query = format!(
                    r#"SELECT rstimestamp as timestamp, price FROM (
            SELECT rstimestamp, max(timestamp), price FROM (
                SELECT price, (timestamp/{frequency})*{frequency} as rstimestamp, volume, timestamp
                FROM trade
                WHERE timestamp <= ? AND timestamp > ? AND
                security = ?
            )
            GROUP BY rstimestamp ORDER BY timestamp ASC
        ) LIMIT 0,100"#,
                    frequency = frequency
                );
                let mut rows = sqlx::query(query.as_str())
                    .bind(max_timestamp)
                    .bind(min_timestamp)
                    .bind(security)
                    .fetch(db_pool);

                let mut prev_timestamp: Option<i64> = None;
                while let Some(row) = rows.try_next().await.unwrap() {
                    // map the row into a user-defined domain type
                    let Ok::<f64, _>(price) = row.try_get("price") else { continue };
                    let Ok::<i64, _>(timestamp) = row.try_get("timestamp") else { continue };

                    // Fill time series gaps with the previous value
                    if let Some(mut prev_timestamp) = prev_timestamp {
                        while prev_timestamp < timestamp {
                            prev_timestamp += frequency;

                            while sqlx::query(
                                "INSERT INTO resampled_trade (price, security, timestamp)
                    VALUES (?, ?, ?)
                    ON CONFLICT (security, timestamp) DO UPDATE SET price = EXCLUDED.price;
                    ;",
                            )
                            .bind(price)
                            .bind(security)
                            .bind(prev_timestamp)
                            .execute(db_pool)
                            .await
                            .is_err()
                            {
                                println!("Error while inserting data");
                                sleep(Duration::from_millis(50)).await;
                            }
                        }
                    }
                    prev_timestamp = Some(timestamp);

                    while sqlx::query(
                        "INSERT INTO resampled_trade (price, security, timestamp)
                VALUES (?, ?, ?)
                ON CONFLICT (security, timestamp) DO UPDATE SET price = EXCLUDED.price;
                ;",
                    )
                    .bind(price)
                    .bind(security)
                    .bind(timestamp)
                    .execute(db_pool)
                    .await
                    .is_err()
                    {
                        println!("Error while inserting data");
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            });
        }
        join_all(tasks).await;
        interval.tick().await;
    }
}
