# Interview Preparation Guide: Kraken SDK

## ❓ Question: "Walk me through your SDK architecture."

### 🗣️ The "Elevator Pitch" (High Level)

"I architected this SDK as an **asynchronous, event-driven system** using the **Actor Pattern** to handle high-throughput financial data safely and efficiently."

There are **three core components** working in harmony:

1.  **The Facade (`KrakenClient`)**:

    - This is the public API that users interact with.
    - It's lightweight and thread-safe.
    - Its only job is to send **Commands** (like "Subscribe") to the background actor via a channel.

2.  **The Actor (`EventLoop`)**:

    - This is the engine running in the background.
    - It **owns the WebSocket connection** exclusively (which solves the Borrow Checker's "mutable aliasing" problem).
    - It processes incoming raw JSON, validates checksums, and manages the OrderBook state.

3.  **The Communication Layer**:
    - **Input**: `mpsc` channel (Multi-Producer, Single-Consumer) for user commands.
    - **Output**: `broadcast` channel for sending typed events (`Trade`, `BookUpdate`) to one or many listeners (Bots, UIs, Loggers).

---

### 📂 File-by-File Walkthrough

Here is how the codebase maps to that architecture:

#### 1. `src/lib.rs` (The Entry Point)

- **Purpose**: The library root. It exports the modules and sets up the strict compilation rules.
- **Key Insight**: Keeps the public API clean by only re-exporting what the user needs (`KrakenClient`, `KrakenEvent`).

#### 2. `src/client.rs` (The Facade)

- **Purpose**: Implements the `KrakenClient` struct.
- **Key Code**: The `connect()` method which spawns the `tokio::spawn(EventLoop::run(...))`. This is where the background actor starts.

#### 3. `src/events.rs` (The Engine)

- **Purpose**: Contains the `EventLoop` struct and the `run` loop.
- **Key Logic**: The `select!` loop that listens to two things simultaneously:
  1.  Network messages (from WebSocket).
  2.  User commands (from the `mpsc` channel).
- **Why**: This ensures the bot stays responsive to user input even when receiving 1000s of market updates.

#### 4. `src/models.rs` (The Protocol)

- **Purpose**: Strongly typed Rust structs mirroring Kraken's API.
- **Key Tech**: Uses `serde` with custom deserializers to handle Kraken's weird "array-based" JSON format (e.g., `["price", "volume"]` instead of keys). this provides **Zero-Copy** parsing where possible.

#### 5. `src/aggregator.rs` (The Logic Layer)

- **Purpose**: Converting raw `Trade` events into `Candle` (OHLCV) bars.
- **Key Algorithm**: Time-bucketing logic (e.g., `timestamp / 60`). This runs O(1) so it doesn't slow down the stream.

#### 6. `src/auth.rs` (The Security)

- **Purpose**: Handles HMAC-SHA512 signing for private feeds.
- **Key Detail**: Generates the `API-Sign` header exactly as Kraken requires (URI path + SHA256(nonce + post data) + Base64 Secret).
