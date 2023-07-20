use super::AppError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
struct Index {
    number_of_raw_trade_records_stored: i64,
    number_of_resampled_trade_records_stored: i64,
}

pub async fn index(State(db_pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
    let x: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM trades_raw"#)
        .fetch_one(&db_pool)
        .await?;
    let number_of_raw_trade_records_stored = x.0;

    let x: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM trades_resampled"#)
        .fetch_one(&db_pool)
        .await?;
    let number_of_resampled_trade_records_stored = x.0;

    let response = Index {
        number_of_raw_trade_records_stored,
        number_of_resampled_trade_records_stored,
    };

    Ok(Json(response))
}

pub async fn not_found() -> AppError {
    AppError::new("Endpoint not found")
        .with_user_message("Endpoint not found")
        .with_code(StatusCode::NOT_FOUND)
}
