mod routes;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

pub async fn run(database_url: String) -> Result<(), String> {
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url.as_str())
        .await
        .map_err(|e| format!("DB connection failed: {}", e))?;

    let app = routes::create_routes(db_pool).await?;
    let bind_addr = &"0.0.0.0:7500";
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("Failed to parse address: {}", e))?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| format!("Server error: {}", e))?;
    Ok(())
}
