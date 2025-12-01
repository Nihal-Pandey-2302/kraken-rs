use kraken_sdk::{KrakenClient, auth::Authenticator};
use std::env;
use dotenvy::dotenv;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    println!("🔐 Starting Private Feed Example...");

    // 1. Load Credentials
    let api_key = env::var("KRAKEN_API_KEY").expect("KRAKEN_API_KEY must be set");
    let api_secret = env::var("KRAKEN_API_SECRET").expect("KRAKEN_API_SECRET must be set");

    // 2. Get WebSocket Token
    println!("🔑 Fetching WebSocket Token...");
    let auth = Authenticator::new(api_key, api_secret);
    let token = auth.get_ws_token().await?;
    println!("✅ Token received: {}...", &token[0..10]);

    // 3. Connect and Subscribe
    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;

    // Subscribe to "ownTrades" (Private Channel)
    println!("📡 Subscribing to 'ownTrades'...");
    client.subscribe(vec![], "ownTrades", Some(token)).await?;

    // 4. Print Events
    while let Ok(event) = rx.recv().await {
        println!("Received: {:?}", event);
    }

    Ok(())
}
