use kraken_sdk::{KrakenClient, models::KrakenEvent};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Custom Handler Example...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;
    client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await?;

    // Example of a custom event processing loop
    while let Ok(event) = rx.recv().await {
        match event {
            KrakenEvent::Heartbeat(_) => {
                // Ignore heartbeats to keep logs clean
            }
            KrakenEvent::SystemStatus(status) => {
                if status.status != "online" {
                    warn!("System is not online: {:?}", status);
                }
            }
            KrakenEvent::Data(_) => {
                // Use the helper to check for trades
                if let Some(trade) = event.clone().try_into_trade_data() {
                    info!("Trade Event: {:?}", trade.pair);
                }
                // Or check for orderbook
                else if let Some(book) = event.try_into_orderbook_data() {
                    info!("Book Event: {:?}", book.pair);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
