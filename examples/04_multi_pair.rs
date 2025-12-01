use kraken_sdk::{KrakenClient, models::KrakenEvent};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Multi-Pair Tracker...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;

    // Subscribe to multiple pairs at once
    let pairs = vec![
        "XBT/USD".to_string(),
        "ETH/USD".to_string(),
        "SOL/USD".to_string(),
        "XRP/USD".to_string(),
    ];
    
    client.subscribe(pairs.clone(), "trade", None).await?;
    info!("Subscribed to: {:?}", pairs);

    while let Ok(event) = rx.recv().await {
        if let Some(trade) = event.try_into_trade_data() {
            info!("[{}] New Trade: {}", trade.pair, trade.data[0].price);
        }
    }

    Ok(())
}
