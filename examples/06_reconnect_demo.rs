use kraken_sdk::{models::KrakenEvent, KrakenClient};

use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Reconnect Demo...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    client.connect().await?;

    // Subscribe to a pair
    client
        .subscribe(vec!["XBT/USD".to_string()], "trade", None)
        .await?;

    // In a real test, you would manually disconnect your internet or kill the WS connection
    // Here we just listen and print, demonstrating that the client stays alive.
    info!("Listening for events. Try disconnecting your internet!");

    while let Ok(event) = rx.recv().await {
        if let Some(trade) = event.clone().try_into_trade_data() {
            info!("Trade: {} trades on {}", trade.data.len(), trade.pair);
        } else if let KrakenEvent::SystemStatus(status) = event {
            warn!("System Status: {:?}", status);
        }
    }

    Ok(())
}
