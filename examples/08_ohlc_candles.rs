use kraken_sdk::{aggregator::TradeAggregator, KrakenClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    // Connect and subscribe to XBT/USD trades
    client.connect().await?;
    client
        .subscribe(vec!["XBT/USD".to_string()], "trade", None)
        .await?;

    println!("🕯️  Starting Candle Aggregator (1-minute candles)...");
    println!("Waiting for trades...");

    // Aggregate into 60-second candles
    let mut aggregator = TradeAggregator::new(60);

    loop {
        match rx.recv().await {
            Ok(event) => {
                if let Some(trade_data) = event.try_into_trade_data() {
                    for trade in trade_data.data {
                        // Check if a candle closed
                        // In a real app, you'd use the trade time, but for this demo we use the trade time too.
                        let trade_time = trade.time.parse::<f64>().unwrap_or(0.0);

                        if let Some(candle) = aggregator.check_flush(trade_time) {
                            println!(
                                "🔥 NEW CANDLE [{}]: O: {:.2} H: {:.2} L: {:.2} C: {:.2} V: {:.4}",
                                candle.start_time,
                                candle.open,
                                candle.high,
                                candle.low,
                                candle.close,
                                candle.volume
                            );
                        }

                        // Update aggregator
                        aggregator.update(&trade);
                    }
                }
            }
            Err(e) => eprintln!("Error receiving event: {}", e),
        }
    }
}
