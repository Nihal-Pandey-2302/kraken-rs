# 🚀 Hackathon Submission Details

Copy-paste these fields into the submission form.

## 1. Project Name

**Kraken Rust SDK**

## 2. Short Tagline (Elevator Pitch)

**A high-performance, strictly-typed Async Rust SDK for Kraken with a built-in Terminal UI (648k msg/sec).**

## 3. Project Description (Long)

**The Problem:**
Building reliable trading systems on raw WebSockets is painful. Dynamic languages like Python are prone to runtime errors when API schemas change, and handling millions of messages per hour requires robust infrastructure.

**The Solution:**
The **Kraken Rust SDK** is a production-grade library designed for high-frequency trading. It leverages Rust's **Actor Pattern** (via Tokio) to decouple the event loop from application logic, ensuring zero-latency handling of market data.

**Key Features:**

- **Performance:** Benchmarked at **~648,000 messages/second** (vs ~602k for Python), using **Zero-Copy Deserialization** (Serde).
- **Grandmaster TUI:** A full-featured Terminal User Interface (built with `ratatui`) to visualize Order Books, Liquidity depth, and Real-time Candles.
- **Correctness:** Implements **CRC32 Checksum Validation** to mathematically verify order book integrity on every update.
- **Reliability:** Deterministic state machine for automatic reconnection and subscription management.

**Why this wins:**
Unlike other wrappers that just expose raw JSON, this SDK provides a **turnkey application** (the TUI) and **verified** private feed authentication. It transforms Kraken's raw firehose into a safe, typed stream of events.

## 4. Tech Stack / Built With

- **Language:** Rust 🦀
- **Async Runtime:** Tokio
- **Networking:** Tokio-Tungstenite (WebSockets), Reqwest (REST)
- **Parsing:** Serde, Serde-Json (Zero-copy)
- **UI:** Ratatui, Crossterm
- **Cryptography:** HMAC-SHA512, CRC32Fast

## 5. Repository URL

`https://github.com/Nihal-Pandey-2302/kraken-rs`

## 6. Demo Video URL

https://youtu.be/hQP03oT1gkY

## 7. YouTube Metadata (Copy-Paste this)

**Title Options:**

- **Option 1 (Clicky):** I Built a High-Frequency Trading Engine in Rust (Kraken Forge)
- **Option 2 (Descriptive):** Building a 648k msg/sec Kraken SDK in Rust + Terminal UI
- **Option 3 (Professional):** Kraken Rust SDK: Production-Grade Async Architecture & TUI

**Description:**
Submitting to the **Kraken Forge Hackathon (Track 1)**.
This is a production-grade, strictly-typed Rust SDK for the Kraken WebSocket API, capable of processing 648,000 messages per second.

Unlike standard API wrappers, this project includes a full **"Grandmaster" Terminal UI** for real-time order book visualization, analytics, and algorithmic execution.

**🔗 Repository:** https://github.com/Nihal-Pandey-2302/kraken-rs

**⚡ Key Features:**

- **Performance:** Zero-Copy Deserialization (~648k msg/s).
- **Safety:** The Actor Pattern (Tokio) & Lock-free Channels.
- **Visualization:** Full TUI with Sparklines & Liquidity Depth.
- **Reliability:** CRC32 Checksum Validation & Auto-Reconnect.

**⏱️ Timestamps:**
0:00 - The Engineering Challenge
0:45 - Async Architecture (Actor Pattern)
1:30 - The "Grandmaster" Terminal UI
2:45 - Algorithmic Trading Bot
3:15 - Reliability & Checksums
3:45 - Conclusion

#Rust #Kraken #HighFrequencyTrading #SystemsEngineering #TUI #Hackathon

## 8. Code Snippets (For Gallery/Images)

**Snippet 1: The "Simple" Start (Carbon.now.sh recommendation)**
_Shows how easy it is to use._

```rust
// 1. Connect
let client = KrakenClient::new();
client.connect().await?;

// 2. Subscribe (One-line)
client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await?;

// 3. Profit
while let Ok(event) = rx.recv().await {
    println!("Received: {:?}", event);
}
```

**Snippet 2: The "Type Safety" Flex**
_Shows why Rust is superior to Python._

```rust
// Strictly Typed Events (Zero Runtime Errors)
if let Some(book) = event.try_into_orderbook_data() {
    if book.validate_checksum() {
        println!("✅ Validated Checksum: {:08x}", book.checksum);
        engine.update_book(book);
    } else {
        panic!("❌ Integrity Failure! Halt Trading!");
    }
}
```
