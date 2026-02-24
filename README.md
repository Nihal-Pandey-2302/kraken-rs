
# Kraken Low-Latency WebSocket Ingestion & Trading SDK (Rust)

High-performance asynchronous Rust infrastructure for real-time market data ingestion, deterministic processing, and low-latency trading systems.

Designed as a production-style system to explore reliability, throughput, and fault tolerance in real-time trading environments.

While structured as an SDK, this project was primarily built as an exploration of real-time ingestion architecture and reliability guarantees in Rust — focusing on deterministic processing, backpressure control, and crash-safe recovery.

[![Watch the Demo](https://img.shields.io/badge/Watch_Demo-FF0000?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/hQP03oT1gkY)

---
## 🚀 Why This Project Exists

Most WebSocket SDKs focus only on connectivity.

Real trading and market data systems must handle:
- deterministic ordering
- burst traffic
- reconnect recovery
- state consistency
- multi-consumer pipelines
- backpressure

This project was built to simulate production-grade ingestion infrastructure rather than a simple API wrapper.

---

## 🧠 System Design Goals

- Deterministic event ordering
- Crash-safe state rebuild
- Backpressure-aware pipelines
- Zero-copy parsing where possible
- Multi-consumer broadcast architecture
- Minimal allocation overhead
- Reconnect & resubscribe resilience

This is closer to a **real trading ingestion engine** than a simple SDK.

---

## ⚡ Performance

Benchmarked against Kraken WebSocket feeds.

| Metric | Result |
|--------|-------|
Throughput | ~648k msgs/sec (local benchmark) |
Latency | ~1–2µs internal parsing |
Allocation Strategy | Buffer reuse + minimal allocations |
Architecture | Async actor-style pipeline |

Performance focus:
- zero-copy parsing
- buffer reuse
- static dispatch via generics
- bounded channels to prevent memory blowup

---
## 🧪 Benchmark Methodology

Throughput and latency numbers were measured under controlled local conditions:

- Replay-based WebSocket message ingestion
- Parsing benchmarked independently from network I/O
- Allocation profiling during sustained load
- Async task isolation to avoid benchmark distortion
- Observed behavior under synthetic burst traffic

The focus was not just peak throughput, but predictable behavior under stress and reconnect conditions.
---
## Architecture Overview
Async event-driven ingestion pipeline with deterministic ordering, local orderbook reconstruction, and checksum validation.

![Kraken Ingestion Architecture](https://raw.githubusercontent.com/Nihal-Pandey-2302/kraken-rs/refs/heads/main/screenshots/arch.png)

Key design decisions:

### Actor-style pipeline
Separated components:
- WebSocket reader task
- parser/decoder
- state manager
- broadcast system

Prevents head-of-line blocking and improves isolation.

### Deterministic processing
Single-writer state update model ensures:
- predictable ordering
- no race conditions in orderbook
- reproducible rebuild

### Backpressure handling
Bounded channels prevent:
- memory explosion
- slow consumer collapse
- unbounded buffering

### Reconnect & recovery
On disconnect:
- reconnect
- resubscribe
- rebuild state from snapshot
- resume streaming
---

## 🧨 Failure Scenarios Considered

The system was designed with explicit failure modes in mind:

- WebSocket disconnect mid-stream
- Out-of-order orderbook deltas
- Checksum mismatch indicating state corruption
- Slow consumer blocking the event loop
- Burst traffic during reconnect
- Sustained high-throughput memory pressure

Handling strategies:

- Automatic reconnect + resubscribe logic
- Snapshot refetch on checksum mismatch
- Single-writer state engine for deterministic rebuild
- Bounded channels to prevent memory explosion
- Explicit resynchronization path on detected inconsistency
---
## 🎨 Integrated Terminal Dashboard (Validation Layer)

Full-featured real-time terminal dashboard built to validate pipeline behavior under load.

![Market Tab](kraken.gif)

- **Real-time Order Book** with Liquidity Visualization
- **Live Analytics** (OHLCV Candles & SMA)
- **Whale Alerts** & **Latency Monitor**

---
## 🔧 Core Features

- Strongly typed message parsing (Serde)
- Local orderbook state engine
- CRC32 checksum validation
- Auto reconnect & resync
- Multi-consumer broadcast via `tokio::broadcast`
- Dynamic subscription management
- Private feed auth support

---

## 📊 Engineering Focus Areas

This project explores:

- Low-latency Rust async systems
- Real-time ingestion architecture
- Orderbook state consistency
- Idempotent event processing
- Multi-stream WebSocket handling
- Fault-tolerant reconnect logic


Run with Docker:

```

docker compose run --rm --build kraken-tui

````

---
## 📂 Examples

We provide several examples to get you started:

### 🟢 Basics

- **[01_basic_subscribe.rs](examples/01_basic_subscribe.rs)**: Simple trade subscription.
- **[04_multi_pair.rs](examples/04_multi_pair.rs)**: Subscribes to multiple pairs (BTC, ETH, SOL, XRP).
- **[06_reconnect_demo.rs](examples/06_reconnect_demo.rs)**: Demonstrates the auto-reconnection logic.

### 🟡 Advanced

- **[02_orderbook_tracker.rs](examples/02_orderbook_tracker.rs)**: Tracks the full order book (snapshot + updates).
- **[03_trade_monitor.rs](examples/03_trade_monitor.rs)**: Monitors trades and alerts on "whale" transactions.
- **[05_custom_handler.rs](examples/05_custom_handler.rs)**: Shows how to handle different event types manually.

### 🔴 Grandmaster Demos

- **[07_terminal_ui.rs](examples/07_terminal_ui.rs)**: **The "Pro" Terminal**. Full TUI with Charts, Sparklines, and Analytics.
- **[08_ohlc_candles.rs](examples/08_ohlc_candles.rs)**: Real-time aggregation of trades into OHLCV candles.
- **[09_private_feed.rs](examples/09_private_feed.rs)**: Authenticated WebSocket subscriptions using HMAC-SHA512.
- **[10_simple_bot.rs](examples/10_simple_bot.rs)**: **Algorithmic Trading**. SMA Crossover strategy relying on the SDK's signals.
---
## 📦 Example Usage

```rust
use kraken_sdk::KrakenClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    client.connect().await?;
    client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await?;

    while let Ok(event) = rx.recv().await {
        println!("{:?}", event);
    }

    Ok(())
}
````
---

## 📈 Scaling Considerations

Current implementation is optimized for single-node, low-latency processing.

Explored scaling strategies include:

- Sharding by trading pair across async tasks
- Partitioned event routing to reduce contention
- Multi-process ingestion with Kafka/NATS fanout
- Externalized persistence layer for distributed consumers

The architecture is intentionally structured to allow evolution toward multi-core and multi-node deployments.
---

## 🏁 Why This Matters

In real-time trading systems, failure rarely occurs at peak throughput.
It occurs during reconnect bursts, inconsistent state rebuild, and slow consumer pressure.

This project was built to deeply understand those failure modes and design reliable ingestion pipelines in Rust.

---

## 👤 Author

Nihal Pandey
Backend Infrastructure Engineer (Rust, Distributed Systems, Real-time Data)

Focus areas:

* high-performance ingestion systems
* async/concurrent Rust services
* distributed backend architecture
* trading & blockchain data infrastructure

Open to:
Remote backend / infrastructure roles
Contract → full-time opportunities

Portfolio: [https://portfolio-alpha-black-q6u25stswg.vercel.app/](https://portfolio-alpha-black-q6u25stswg.vercel.app/)


