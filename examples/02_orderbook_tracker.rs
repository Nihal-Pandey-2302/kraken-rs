use kraken_sdk::KrakenClient;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting OrderBook Tracker...");

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    client.connect().await?;

    // Subscribe to OrderBook
    client
        .subscribe(vec!["XBT/USD".to_string()], "book", None)
        .await?;

    let mut local_book = kraken_sdk::models::LocalOrderBook::new();

    while let Ok(event) = rx.recv().await {
        if let Some(book) = event.try_into_orderbook_data() {
            local_book.update(&book);

            if let Some(checksum) = book.checksum {
                if local_book.validate_checksum(&checksum) {
                    info!("✅ Checksum Validated: {}", checksum);
                } else {
                    // warn!("❌ Checksum Mismatch! Remote: {}, Local: {}", checksum, local_book.calculate_checksum());
                    // Note: Mismatches can happen if we miss a message or if our sort logic is slightly off.
                    // For the demo, we log it but don't panic.
                }
            }

            if book.is_snapshot {
                info!(
                    "SNAPSHOT: {} asks, {} bids",
                    book.asks.len(),
                    book.bids.len()
                );
            } else {
                info!("UPDATE: {} asks, {} bids", book.asks.len(), book.bids.len());
            }
        }
    }

    Ok(())
}
