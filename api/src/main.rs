use dotenvy::var;
use rust_trading_platform_api::run;

#[tokio::main]
async fn main() -> Result<(), String> {
    let database_uri =
        var("DATABASE_URL").map_err(|e| format!("Failed to get DATABASE_URL: {}", e))?;
    run(database_uri).await
}
