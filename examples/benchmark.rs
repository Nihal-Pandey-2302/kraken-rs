use std::fs;
use std::time::Instant;
use kraken_sdk::models::KrakenEvent;

fn main() {
    // Setup: Load data and prepare as individual message strings (simulating WS frames)
    let content = fs::read_to_string("examples/benchmark_data.json").expect("Failed to read benchmark_data.json");
    let raw_msgs: Vec<serde_json::Value> = serde_json::from_str(&content).expect("Failed to parse JSON array");
    let msgs: Vec<String> = raw_msgs.iter().map(|v| v.to_string()).collect();

    println!("🦀 Benchmarking Rust SDK (Strict Types) with {} messages...", msgs.len());

    let start = Instant::now();
    let mut count = 0;
    for msg in &msgs {
        // The core workload: Parse string -> Strongly Typed Struct
        let event: KrakenEvent = serde_json::from_str(msg).unwrap();
        
        // Access data to ensure optimizer doesn't kill it
        match event {
            KrakenEvent::Data(_d) => { count += 1; },
            _ => { count += 1; }
        }
    }
    let duration = start.elapsed();

    println!("✅ Processed {} messages in {:.4?}", count, duration);
    println!("🚀 Throughput: {:.2} msgs/sec", count as f64 / duration.as_secs_f64());
}
