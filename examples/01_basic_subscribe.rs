use kraken_sdk::{KrakenClient, models::KrakenEvent};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Basic Subscribe Example...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;

    // Subscribe to XBT/USD trades
    client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await?;

    while let Ok(event) = rx.recv().await {
        if let Some(trade) = event.try_into_trade_data() {
            info!("Received {} trades for {}", trade.data.len(), trade.pair);
            for t in trade.data {
                info!("  Price: {}, Vol: {}", t.price, t.volume);
            }
        }
    }

    Ok(())
}
