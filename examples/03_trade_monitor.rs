use kraken_sdk::{KrakenClient, models::KrakenEvent};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Trade Monitor...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;

    client.subscribe(vec!["XBT/USD".to_string(), "ETH/USD".to_string()], "trade", None).await?;

    while let Ok(event) = rx.recv().await {
        if let Some(trade_data) = event.try_into_trade_data() {
            for trade in trade_data.data {
                let price: f64 = trade.price.parse().unwrap_or(0.0);
                let volume: f64 = trade.volume.parse().unwrap_or(0.0);
                let value = price * volume;
                
                if value > 1000.0 {
                     info!("🚨 WHALE ALERT on {}: ${:.2} ({} @ {})", trade_data.pair, value, volume, price);
                } else {
                     info!("Trade on {}: ${:.2}", trade_data.pair, value);
                }
            }
        }
    }

    Ok(())
}
