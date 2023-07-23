use crate::{RESAMPLE_RESOLUTION, SECURITIES};

use futures::{future::join_all, TryStreamExt};
use sqlx::{PgPool, Row};
use tokio::time::{interval, sleep, Duration};

pub async fn resample_trades(db_pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Maximum update resolution in milliseconds
    let mut interval = interval(Duration::from_millis(10));

    // Past recheck leap in milliseconds
    // This is the amount of time in milliseconds that the resampler will
    // recheck for wrong data in the past
    // this is usefull because sometimes, the websocket connection will
    // give really out of order
    let past_recheck_leap = std::cmp::max(2 * RESAMPLE_RESOLUTION, 2000);

    loop {
        let mut tasks = Vec::new();
        for security in SECURITIES {
            tasks.push(async move {
                let max_timestamp: Result<(i64,), _> =
                    sqlx::query_as(r#"SELECT MAX(timestamp) FROM trades_raw WHERE security = $1;"#)
                        .bind(security)
                        .fetch_one(db_pool)
                        .await;

                let Ok(max_timestamp) = max_timestamp else {return};

                let max_timestamp =
                    max_timestamp.0.div_euclid(RESAMPLE_RESOLUTION) * RESAMPLE_RESOLUTION;

                let min_timestamp: (i64,) = sqlx::query_as(
                    "SELECT MAX(timestamp) FROM trades_resampled WHERE security = $1;",
                )
                .bind(security)
                .fetch_one(db_pool)
                .await
                .unwrap_or_default();

                let min_timestamp = min_timestamp.0 - past_recheck_leap;

                let query = format!(
                    "
SELECT sq2.rstimestamp as timestamp, sq2.price as price FROM (
    SELECT
    ROW_NUMBER() OVER (PARTITION BY sq1.rstimestamp
        ORDER BY sq1.timestamp DESC) AS row_id,
    sq1.rstimestamp as rstimestamp, sq1.price as price FROM (
            SELECT price,
            (timestamp/{resolution})*{resolution}+{resolution} as rstimestamp,
            volume, timestamp
            FROM trades_raw
            WHERE timestamp <= $1 AND timestamp > $2 AND
            security = $3
    ) as sq1
) as sq2 WHERE sq2.row_id = 1",
                    resolution = RESAMPLE_RESOLUTION
                );
                let mut rows = sqlx::query(query.as_str())
                    .bind(max_timestamp)
                    .bind(min_timestamp)
                    .bind(security)
                    .fetch(db_pool);

                let mut prev_timestamp: Option<i64> = None;
                while let Some(row) = rows.try_next().await.unwrap() {
                    let Ok::<f64, _>(price) = row.try_get("price") else { continue };
                    let Ok::<i64, _>(timestamp) = row.try_get("timestamp") else { continue };

                    // Fill time series gaps with the previous value
                    if let Some(mut prev_timestamp) = prev_timestamp {
                        while prev_timestamp < timestamp {
                            prev_timestamp += RESAMPLE_RESOLUTION;

                            while let Err(err) = sqlx::query(
                                "INSERT INTO trades_resampled (price, security, timestamp)
VALUES ($1, $2, $3)
ON CONFLICT (security, timestamp) DO UPDATE SET price = EXCLUDED.price;
;",
                            )
                            .bind(price)
                            .bind(security)
                            .bind(prev_timestamp)
                            .execute(db_pool)
                            .await
                            {
                                println!(
                                    "Error while inserting data into trades_resampled table: {:?}",
                                    err
                                );
                                sleep(Duration::from_millis(50)).await;
                            }
                        }
                    }
                    prev_timestamp = Some(timestamp);

                    while let Err(err) = sqlx::query(
                        "INSERT INTO trades_resampled (price, security, timestamp)
VALUES ($1, $2, $3)
ON CONFLICT (security, timestamp) DO UPDATE SET price = EXCLUDED.price;
;",
                    )
                    .bind(price)
                    .bind(security)
                    .bind(timestamp)
                    .execute(db_pool)
                    .await
                    {
                        println!("Error while inserting data into trades_resampled table: {err}");
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            });
        }
        join_all(tasks).await;
        interval.tick().await;
    }
}
