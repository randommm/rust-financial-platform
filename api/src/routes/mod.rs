mod error_handling;
mod pages;
use axum::{extract::FromRef, routing::get, Router};
use error_handling::AppError;
use sqlx::SqlitePool;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

pub async fn create_routes(db_pool: SqlitePool) -> Result<Router, String> {
    let app_state = AppState { db_pool };

    let api = Router::new()
        .route("/", get(pages::index))
        .route("/resampled_trades", get(pages::get_resampled_trades))
        .route("/securities", get(pages::list_securities))
        .with_state(app_state.clone());

    Ok(Router::new()
        .nest("/api/v1", api)
        .fallback(get(pages::not_found)))
}
