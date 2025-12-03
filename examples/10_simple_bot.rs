use kraken_sdk::{aggregator::TradeAggregator, KrakenClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    
    // 1. Setup Client
    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    // 2. Connect & Subscribe
    println!("🤖 Simple Bot: Connecting to Kraken...");
    client.connect().await?;
    client
        .subscribe(vec!["XBT/USD".to_string()], "trade", None)
        .await?;
    println!("✅ Connected! Subscribed to XBT/USD trades.");

    // 3. Strategy State
    // We'll use 10-second candles for this demo (faster feedback)
    let mut aggregator = TradeAggregator::new(10);
    let mut candles = Vec::new();

    // SMA Periods
    let fast_period = 5;
    let slow_period = 20;

    println!(
        "📈 Strategy: SMA Crossover (Fast={}, Slow={})",
        fast_period, slow_period
    );
    println!("Waiting for candle data...");

    // 4. Event Loop
    while let Ok(event) = rx.recv().await {
        if let Some(trade) = event.try_into_trade_data() {
            for t in trade.data {
                // Update Aggregator
                let trade_time = t.time.parse::<f64>().unwrap_or(0.0);

                // Check if a new candle is formed
                if let Some(candle) = aggregator.check_flush(trade_time) {
                    candles.push(candle.clone());

                    // Keep history manageable
                    if candles.len() > slow_period + 1 {
                        candles.remove(0);
                    }

                    // Calculate Indicators
                    if candles.len() >= slow_period {
                        let fast_sma = calculate_sma(&candles, fast_period);
                        let slow_sma = calculate_sma(&candles, slow_period);

                        let price = candle.close;

                        println!(
                            "🕯️ Candle Closed: ${:.2} | SMA({}): {:.2} | SMA({}): {:.2}",
                            price, fast_period, fast_sma, slow_period, slow_sma
                        );

                        // Signal Logic
                        if fast_sma > slow_sma {
                            println!("🚀 BUY SIGNAL (Fast > Slow)");
                        } else if fast_sma < slow_sma {
                            println!("🔻 SELL SIGNAL (Fast < Slow)");
                        } else {
                            println!("⚖️  HOLD");
                        }
                    } else {
                        println!("⏳ Building History: {}/{}", candles.len(), slow_period);
                    }
                }

                // Add trade to current candle
                aggregator.update(&t);
            }
        }
    }

    Ok(())
}

fn calculate_sma(candles: &[kraken_sdk::models::Candle], period: usize) -> f64 {
    if candles.len() < period {
        return 0.0;
    }
    let start = candles.len() - period;
    let sum: f64 = candles[start..].iter().map(|c| c.close).sum();
    sum / period as f64
}
