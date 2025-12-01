use eyre::Result;
use kraken_sdk::{KrakenClient, models::KrakenEvent};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    println!("Starting Kraken SDK Example...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();
    
    client.connect().await?;

    // Dynamic subscription
    client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await?;
    client.subscribe(vec!["XBT/USD".to_string()], "book", None).await?;

    // Consume events
    while let Ok(event) = rx.recv().await {
        // We clone event because try_into consumes it, and we might want to try multiple things
        // Or better, we match on the result of the first try, then the second.
        
        // Since `try_into_*` consumes self, we can't chain them easily on the same variable without cloning.
        // But `KrakenEvent` is Clone.
        
        let event_clone = event.clone();
        
        if let Some(trade_data) = event_clone.try_into_trade_data() {
             info!(">>> Trade on {}: {} trades", trade_data.pair, trade_data.data.len());
        } else if let Some(ob_data) = event.try_into_orderbook_data() {
             let type_str = if ob_data.is_snapshot { "SNAPSHOT" } else { "UPDATE" };
             info!(">>> OrderBook {} on {}: {} asks, {} bids", type_str, ob_data.pair, ob_data.asks.len(), ob_data.bids.len());
             if !ob_data.asks.is_empty() {
                 info!("    Top Ask: Price {}, Vol {}", ob_data.asks[0].price, ob_data.asks[0].volume);
             }
             if !ob_data.bids.is_empty() {
                 info!("    Top Bid: Price {}, Vol {}", ob_data.bids[0].price, ob_data.bids[0].volume);
             }
        } else {
            // info!("Received other event: {:?}", event);
        }
    }

    Ok(())
}
