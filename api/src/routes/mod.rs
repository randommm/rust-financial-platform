mod error_handling;
mod pages;
use error_handling::AppError;

use axum::{extract::FromRef, routing::get, Router};
use sqlx::SqlitePool;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

pub async fn create_routes(db_pool: SqlitePool) -> Result<Router, String> {
    let app_state = AppState { db_pool };

    Ok(Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/", get(pages::index))
                .with_state(app_state.clone()),
        )
        .fallback(get(pages::not_found)))
}
