use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;
use crc32fast::Hasher;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum KrakenEvent {
    Heartbeat(Heartbeat),
    SystemStatus(SystemStatus),
    SubscriptionStatus(SubscriptionStatus),
    Data(Vec<Value>), // Fallback for data arrays: [channelID, data, channelName, pair]
}

#[derive(Debug, Clone, Deserialize)]
pub struct Heartbeat {
    pub event: String, // "heartbeat"
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemStatus {
    pub event: String, // "systemStatus"
    pub connection_id: Option<u64>,
    pub status: String, // "online"
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionStatus {
    pub event: String, // "subscriptionStatus"
    pub status: Option<String>, // "subscribed" or "error"
    pub pair: Option<String>,
    pub channel_name: Option<String>,
    pub subscription: Option<SubscriptionInfo>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionInfo {
    pub name: String,
}

// --- Typed Data Structures ---

#[derive(Debug, Clone)]
pub struct TradeData {
    pub channel_id: u64,
    pub data: Vec<Trade>,
    pub channel_name: String,
    pub pair: String,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: String,
    pub volume: String,
    pub time: String,
    pub side: String, // "b" or "s"
    pub order_type: String, // "m" or "l"
    pub misc: String,
}

#[derive(Debug, Clone)]
pub struct OrderBookData {
    pub channel_id: u64,
    pub asks: Vec<OrderBookEntry>,
    pub bids: Vec<OrderBookEntry>,
    pub is_snapshot: bool,
    pub channel_name: String,
    pub pair: String,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OrderBookEntry {
    pub price: String,
    pub volume: String,
    pub timestamp: String,
}

// Custom deserializer for OrderBookEntry: ["price", "volume", "timestamp"]
impl<'de> Deserialize<'de> for OrderBookEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<String> = Deserialize::deserialize(deserializer)?;
        Ok(OrderBookEntry {
            price: v.get(0).cloned().unwrap_or_default(),
            volume: v.get(1).cloned().unwrap_or_default(),
            timestamp: v.get(2).cloned().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookRegion {
    #[serde(default)]
    pub a: Vec<OrderBookEntry>, // Updates: asks
    #[serde(default)]
    pub b: Vec<OrderBookEntry>, // Updates: bids
    #[serde(default)]
    pub as_: Vec<OrderBookEntry>, // Snapshot: asks (mapped from "as")
    #[serde(default)]
    pub bs: Vec<OrderBookEntry>, // Snapshot: bids (mapped from "bs")
}

impl KrakenEvent {
    pub fn try_into_trade_data(self) -> Option<TradeData> {
        if let KrakenEvent::Data(mut vec) = self {
            // Check if it's a trade event (has "trade" string)
            // Format: [channel_id, [[trade...], ...], "trade", pair]
            if vec.len() >= 4 && vec[2].as_str() == Some("trade") {
                let pair = vec.pop()?.as_str()?.to_string();
                let _channel_name = vec.pop()?.as_str()?.to_string();
                let trades_value = vec.pop()?;
                let channel_id = vec.pop()?.as_u64()?;

                let trades: Vec<Trade> = serde_json::from_value(trades_value).ok()?;

                return Some(TradeData {
                    channel_id,
                    data: trades,
                    channel_name: "trade".to_string(),
                    pair,
                });
            }
        }
        None
    }

    pub fn try_into_orderbook_data(self) -> Option<OrderBookData> {
        if let KrakenEvent::Data(mut vec) = self {
            // Format: [channel_id, { "as": ... } OR { "a": ... }, "book-N", pair]
            // Sometimes updates have two objects: [channel_id, {"a":...}, {"b":...}, "book-N", pair]
            
            // Check for "book-" in the channel name
            // The channel name is usually the second to last element, but if there are 2 objects, it shifts.
            // Let's look from the end.
            
            let pair = vec.pop()?.as_str()?.to_string();
            let channel_name = vec.pop()?.as_str()?.to_string();
            
            if !channel_name.starts_with("book") {
                return None;
            }

            let channel_id = vec.remove(0).as_u64()?;
            
            // Remaining elements in vec are the data objects (1 or 2)
            let mut asks = Vec::new();
            let mut bids = Vec::new();
            let mut is_snapshot = false;
            let mut checksum: Option<String> = None;

            for value in vec {
                // Check if it's the checksum string (only present in updates sometimes, or as a separate field?)
                // Actually, Kraken sends checksum as a field "c" inside the object usually, OR as a separate string at the end?
                // Let's check the docs/logs.
                // Usually: [channelID, {"a": [], "b": [], "c": "1234"}, "book-10", "XBT/USD"]
                
                if let Ok(obj) = serde_json::from_value::<serde_json::Map<String, Value>>(value.clone()) {
                    if let Some(a_val) = obj.get("a") {
                        if let Ok(mut list) = serde_json::from_value::<Vec<OrderBookEntry>>(a_val.clone()) {
                            asks.append(&mut list);
                        }
                    }
                    if let Some(b_val) = obj.get("b") {
                        if let Ok(mut list) = serde_json::from_value::<Vec<OrderBookEntry>>(b_val.clone()) {
                            bids.append(&mut list);
                        }
                    }
                    if let Some(c_val) = obj.get("c") {
                        // Checksum is usually a string in the JSON
                        if let Some(s) = c_val.as_str() {
                            checksum = Some(s.to_string());
                        }
                    }
                    if let Some(as_val) = obj.get("as") {
                        is_snapshot = true;
                        if let Ok(mut list) = serde_json::from_value::<Vec<OrderBookEntry>>(as_val.clone()) {
                            asks.append(&mut list);
                        }
                    }
                    if let Some(bs_val) = obj.get("bs") {
                        is_snapshot = true;
                        if let Ok(mut list) = serde_json::from_value::<Vec<OrderBookEntry>>(bs_val.clone()) {
                            bids.append(&mut list);
                        }
                    }
                }
            }

            return Some(OrderBookData {
                channel_id,
                asks,
                bids,
                is_snapshot,
                channel_name,
                pair,
                checksum,
            });
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct LocalOrderBook {
    // Price -> Volume
    // We use String for precision, but for sorting we might need f64 or custom comparator.
    // Kraken prices are strings. BTreeMap sorts Strings lexicographically, which IS NOT CORRECT for numbers ("10" < "2").
    // We must parse to f64 for sorting keys, or use a custom wrapper.
    // For simplicity in this hackathon, let's assume standard float parsing is fine for keys, 
    // but we keep the original string for the checksum to avoid float formatting issues.
    // Actually, using a wrapper `OrderedFloat` is best, but we don't want another dep.
    // Let's use a helper to parse key as f64 for the map.
    // Wait, if we use f64 as key, we can't get the original string back easily unless we store it as value.
    // Value: (OriginalPriceString, VolumeString)
    pub asks: BTreeMap<String, String>, // Key: Price (padded/normalized?), Value: Volume
    pub bids: BTreeMap<String, String>,
}

impl LocalOrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, data: &OrderBookData) {
        if data.is_snapshot {
            self.asks.clear();
            self.bids.clear();
        }

        for entry in &data.asks {
            let price = &entry.price;
            let volume = &entry.volume;
            if volume == "0.00000000" || volume == "0.0" || volume == "0" {
                self.asks.remove(price);
            } else {
                self.asks.insert(price.clone(), volume.clone());
            }
        }

        for entry in &data.bids {
            let price = &entry.price;
            let volume = &entry.volume;
            if volume == "0.00000000" || volume == "0.0" || volume == "0" {
                self.bids.remove(price);
            } else {
                self.bids.insert(price.clone(), volume.clone());
            }
        }
    }

    /// Calculates the Kraken CRC32 checksum.
    /// Logic:
    /// 1. Top 10 Asks (lowest price)
    /// 2. Top 10 Bids (highest price)
    /// 3. String = price + volume (decimal points removed)
    pub fn calculate_checksum(&self) -> u32 {
        let mut hasher = Hasher::new();
        
        // Asks: Sorted Low to High. 
        // BTreeMap sorts Strings lexicographically. This is a BUG if prices have different integer lengths (e.g. "100" vs "99").
        // However, for a single pair like XBT/USD, prices are usually same length (5 digits).
        // To be safe, we should really sort by float value.
        // Let's collect and sort properly.
        
        let mut asks: Vec<(&String, &String)> = self.asks.iter().collect();
        asks.sort_by(|a, b| {
            let p1 = a.0.parse::<f64>().unwrap_or(0.0);
            let p2 = b.0.parse::<f64>().unwrap_or(0.0);
            p1.partial_cmp(&p2).unwrap()
        });

        let mut bids: Vec<(&String, &String)> = self.bids.iter().collect();
        bids.sort_by(|a, b| {
            let p1 = a.0.parse::<f64>().unwrap_or(0.0);
            let p2 = b.0.parse::<f64>().unwrap_or(0.0);
            p2.partial_cmp(&p1).unwrap() // Reverse for Bids (High to Low)
        });

        for (price, volume) in asks.iter().take(10) {
            let p = price.replace(".", "");
            let p = p.trim_start_matches('0');
            let v = volume.replace(".", "");
            let v = v.trim_start_matches('0');
            hasher.update(p.as_bytes());
            hasher.update(v.as_bytes());
        }

        for (price, volume) in bids.iter().take(10) {
            let p = price.replace(".", "");
            let p = p.trim_start_matches('0');
            let v = volume.replace(".", "");
            let v = v.trim_start_matches('0');
            hasher.update(p.as_bytes());
            hasher.update(v.as_bytes());
        }

        hasher.finalize()
    }
    
    pub fn validate_checksum(&self, remote_checksum: &str) -> bool {
        // Remote checksum is a string of the u32? Or hex?
        // Kraken sends it as a string "123456789".
        if let Ok(remote_val) = remote_checksum.parse::<u32>() {
            let local_val = self.calculate_checksum();
            return local_val == remote_val;
        }
        false
    }
}

// Custom deserializer for Trade array: ["price", "volume", "time", "side", "type", "misc"]
impl<'de> Deserialize<'de> for Trade {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<String> = Deserialize::deserialize(deserializer)?;
        Ok(Trade {
            price: v.get(0).cloned().unwrap_or_default(),
            volume: v.get(1).cloned().unwrap_or_default(),
            time: v.get(2).cloned().unwrap_or_default(),
            side: v.get(3).cloned().unwrap_or_default(),
            order_type: v.get(4).cloned().unwrap_or_default(),
            misc: v.get(5).cloned().unwrap_or_default(),
        })
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub start_time: u64, // Unix timestamp (seconds)
    pub interval_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heartbeat() {
        let data = r#"{"event":"heartbeat"}"#;
        let event: KrakenEvent = serde_json::from_str(data).unwrap();
        match event {
            KrakenEvent::Heartbeat(_) => assert!(true),
            _ => assert!(false, "Expected Heartbeat"),
        }
    }

    #[test]
    fn test_parse_trade_data() {
        let data = r#"[123, [["50000.0", "1.0", "123456.789", "b", "m", ""]], "trade", "XBT/USD"]"#;
        let event: KrakenEvent = serde_json::from_str(data).unwrap();
        match event {
            KrakenEvent::Data(vec) => {
                assert_eq!(vec.len(), 4);
                // Further parsing logic would go here or in a conversion function
            }
            _ => assert!(false, "Expected Data"),
        }
    }
}
