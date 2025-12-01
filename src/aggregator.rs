use crate::models::{Candle, Trade};

pub struct TradeAggregator {
    interval_seconds: u64,
    current_candle: Option<Candle>,
}

impl TradeAggregator {
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            interval_seconds,
            current_candle: None,
        }
    }

    pub fn update(&mut self, trade: &Trade) {
        let price = trade.price.parse::<f64>().unwrap_or(0.0);
        let volume = trade.volume.parse::<f64>().unwrap_or(0.0);
        let time = trade.time.parse::<f64>().unwrap_or(0.0) as u64;

        // Determine the start time of the candle this trade belongs to
        let candle_start = (time / self.interval_seconds) * self.interval_seconds;

        if let Some(candle) = &mut self.current_candle {
            if candle.start_time == candle_start {
                // Update existing candle
                candle.high = candle.high.max(price);
                candle.low = candle.low.min(price);
                candle.close = price;
                candle.volume += volume;
                return;
            } else {
                // This trade belongs to a new candle (or we missed some, but we assume stream is roughly ordered)
                // In a real system, we might buffer or handle out-of-order trades.
                // For this SDK, we'll just start a new one.
                // The caller should have checked `check_flush` before calling update if they wanted the closed candle.
            }
        }

        // Start new candle
        self.current_candle = Some(Candle {
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            start_time: candle_start,
            interval_seconds: self.interval_seconds,
        });
    }

    /// Checks if the current candle is "done" based on the new time, returning it if so.
    /// This is a simplified logic: we return the *previous* candle if the *new* time belongs to a later interval.
    pub fn check_flush(&mut self, new_trade_time: f64) -> Option<Candle> {
        let time = new_trade_time as u64;
        let new_candle_start = (time / self.interval_seconds) * self.interval_seconds;

        if let Some(candle) = &self.current_candle {
            if new_candle_start > candle.start_time {
                // The time has moved to the next interval. The current candle is closed.
                return self.current_candle.take();
            }
        }
        None
    }
}
