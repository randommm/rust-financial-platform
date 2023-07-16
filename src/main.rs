use dotenvy::var;
use finnhub_rust_platform::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        var("DATABASE_URL").map_err(|e| format!("Failed to get DATABASE_URL: {}", e))?;

    let finnhub_token =
        var("FINNHUB_TOKEN").map_err(|e| format!("Failed to get FINNHUB_TOKEN: {}", e))?;
    let connect_addr = "wss://ws.finnhub.io?token=".to_owned() + &finnhub_token;

    run(connect_addr, database_url).await?;
    Ok(())
}
