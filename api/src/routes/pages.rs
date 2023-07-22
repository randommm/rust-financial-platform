use super::AppError;
use axum::{
    extract::{Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Html, IntoResponse},
};
use axum_extra::response::ErasedJson;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
#[derive(Serialize, Deserialize)]
struct ResponseIndex {
    number_of_raw_trade_records_stored: i64,
    number_of_resampled_trade_records_stored: i64,
}

#[derive(Serialize, Deserialize)]
struct RTQResponse {
    price: f64,
    timestamp: i64,
}

#[derive(Deserialize, Debug)]
pub struct ResampledTradesQuery {
    security: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
    frequency: Option<i64>,
    order: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
}

pub async fn index(State(db_pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let x: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM trades_raw"#)
        .fetch_one(&db_pool)
        .await?;
    let number_of_raw_trade_records_stored = x.0;

    let x: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM trades_resampled"#)
        .fetch_one(&db_pool)
        .await?;
    let number_of_resampled_trade_records_stored = x.0;

    let response = ResponseIndex {
        number_of_raw_trade_records_stored,
        number_of_resampled_trade_records_stored,
    };

    Ok(ErasedJson::pretty(response))
}

pub async fn get_resampled_trades(
    State(db_pool): State<PgPool>,
    Query(query): Query<ResampledTradesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let frequency = query.frequency.unwrap_or(1);
    let order = query.order.unwrap_or("a".to_owned());
    let from = query
        .from
        .map(|x| format!("AND timestamp >= {}", x))
        .unwrap_or("".to_owned());
    let to = query
        .to
        .map(|x| format!("AND timestamp <= {}", x))
        .unwrap_or("".to_owned());

    let Some(security) = query.security else {
        return Err(AppError::new("security must be specified")
            .with_user_message("security must be specified")
            .with_code(StatusCode::BAD_REQUEST));
    };
    if !(1..=50).contains(&per_page) {
        return Err(
            AppError::new("per_page must be greater than 0 and lower than 51")
                .with_user_message("per_page must be greater than 0 and lower than 51")
                .with_code(StatusCode::BAD_REQUEST),
        );
    }
    if page < 1 {
        return Err(AppError::new("page must be greater than 0")
            .with_user_message("page must be greater than 0")
            .with_code(StatusCode::BAD_REQUEST));
    }
    if frequency < 1 {
        return Err(AppError::new("frequency must be greater than 0")
            .with_user_message("frequency must be greater than 0")
            .with_code(StatusCode::BAD_REQUEST));
    }
    let order = if order == "a" {
        "ASC "
    } else if order == "d" {
        "DESC"
    } else {
        return Err(AppError::new("order must be 'a' or 'd'")
            .with_user_message("order must be 'a' or 'd'")
            .with_code(StatusCode::BAD_REQUEST));
    };

    let offset = (page - 1) * per_page;

    let sql_query = format!(
        r#"
        SELECT * FROM
        (
                SELECT
                ROW_NUMBER() OVER (ORDER BY timestamp {order}) AS row_id,price,timestamp
                FROM trades_resampled
                WHERE security = $1
                {from} {to}
                ORDER BY timestamp {order}
        ) as dtable
        WHERE
        (row_id - 1) % {frequency} = 0
        LIMIT {per_page} OFFSET {offset}
        "#,
        from = from,
        to = to,
        order = order,
        offset = offset,
        per_page = per_page,
        frequency = frequency,
    );
    let mut rows = sqlx::query(&sql_query).bind(security).fetch(&db_pool);

    let mut response = Vec::new();
    while let Some(row) = rows.try_next().await? {
        let price = row.try_get("price")?;
        let timestamp = row.try_get("timestamp")?;
        response.push(RTQResponse { price, timestamp });
    }

    Ok(ErasedJson::pretty(response))
}

pub async fn list_securities(State(db_pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let sql_query = "SELECT DISTINCT(security) FROM trades_resampled";
    let mut rows = sqlx::query(sql_query).fetch(&db_pool);
    let mut response = Vec::new();
    while let Some(row) = rows.try_next().await? {
        let security: String = row.try_get("security")?;
        response.push(security);
    }

    Ok(ErasedJson::pretty(response))
}

pub async fn not_found_json() -> AppError {
    AppError::new("Endpoint not found")
        .with_user_message("Endpoint not found")
        .with_code(StatusCode::NOT_FOUND)
}

pub async fn not_found_html() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html(
            r#"<!doctype html>
    <html>
      <head>
      </head>
      <body>
        Page not found. <a href="/docs">Check the API documentation</a>.
      </body>
    </html>"#,
        ),
    )
        .into_response()
}

const SWAGGER_JSON: &str = include_str!("../assets/swagger.json");
const DOCS_HTML: &str = include_str!("../assets/docs.html");
pub async fn swagger_json() -> impl IntoResponse {
    ([(CONTENT_TYPE, "application/json")], SWAGGER_JSON)
}
pub async fn api_docs() -> impl IntoResponse {
    Html(DOCS_HTML)
}
