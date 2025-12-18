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

Here is how the `src` folder implements this architecture:

#### 1. `src/lib.rs` (The Core Hub)

- **Role**: The "Motherboard" of the SDK.
- **What it does**:
  - Defines `KrakenClient` (the public struct you use).
  - **Spawns the Actor**: The `connect()` method contains the `tokio::spawn` block that runs the actual WebSocket loop.
  - **Manages Channels**: Creates the `mpsc` (command) and `broadcast` (event) channels.
- **Key Concept**: It hides the complexity. Users just see `client.subscribe()`, but `lib.rs` is secretly passing messages to a background thread.

#### 2. `src/models.rs` (The Data Protocol)

- **Role**: The "Dictionary" of the SDK.
- **What it does**: Defines every single JSON message Kraken can send as a strict Rust struct (e.g., `TradeData`, `OrderBookData`).
- **Key Concept**: **Zero-Copy Parsing**. we use `serde` to map incoming bytes directly to structs, failing fast if the API changes.

#### 3. `src/auth.rs` (The Security Layer)

- **Role**: The "Bouncer".
- **What it does**: Generates the `API-Sign` header for private feeds.
- **Key Concept**: Crypto math (HMAC-SHA512 + SHA256) encapsulated in a simple `Authenticator` struct.

#### 4. `src/aggregator.rs` (The Business Logic)

- **Role**: The "Processor".
- **What it does**: Takes raw `Trade` events and compiles them into `Candle` (OHLCV) bars.
- **Key Concept**: State management. It buffers trades in memory until a time bucket (e.g., 60s) closes.

#### 5. `examples/` (The Consumers)

- **Role**: The "Proof".
- **What it does**: Shows how to _use_ the SDK. `07_terminal_ui.rs` is the "Flagship" consumer showing all features working together.

---

### 🔗 How They Connect

1.  **User** calls `KrakenClient::new()` (in `lib.rs`).
2.  **User** calls `connect()` -> This spawns the **Actor Loop** (inside `lib.rs`).
3.  **User** calls `subscribe_private()` -> Uses `auth.rs` to sign the token.
4.  **Actor** receives WS message -> Uses `models.rs` to parse it.
5.  **Actor** broadcasts event -> User receives it.
6.  (Optional) **User** passes event to `TradeAggregator` (in `aggregator.rs`) to make candles.

---

## 🧠 Prerequisite Concepts (Study These First)

To really understand this codebase, you need to be comfortable with these 5 Rust concepts:

### 1. **Channels (`mpsc` vs `broadcast`)**

- **Concept**: How threads talk to each other without fighting over memory.
- **In this SDK**:
  - `mpsc` (Multi-Producer, Single-Consumer): Used for **Commands** (You -> Client -> EventLoop). Many parts of your code can send commands, but only the EventLoop listens.
  - `broadcast`: Used for **Events** (EventLoop -> You). One message (e.g., "Price Update") is sent to _all_ active listeners (Logger, UI, Bot).

### 2. **Async/Await & Tokio Tasks**

- **Concept**: Doing many things at once without using many OS threads.
- **In this SDK**: The `EventLoop` runs as a separate **green thread** (`tokio::spawn`). It sleeps efficiently when there's no data, so it uses almost 0% CPU at idle.

### 3. **Serde (Serialization/Deserialization)**

- **Concept**: Turning JSON text into Rust Structs.
- **In this SDK**: We use advanced Serde features like `#[serde(untagged)]` to handle Kraken's messy JSON. Instead of writing `if/else` parsers, we define the _shape_ of the data and let Serde do the hard work.

### 4. **The Actor Pattern**

- **Concept**: A design where a "worker" (Actor) owns a resource (WebSocket) and others only talk to it via messages.
- **In this SDK**: The `EventLoop` is the Actor. It prevents "Race Conditions" because _only_ the EventLoop is allowed to write to the WebSocket.

### 5. **`select!` Loop**

- **Concept**: Listening to multiple channels at the same time.
- **In this SDK**: The heart of the engine. It waits for EITHER a WebSocket message OR a User Command. Whichever arrives first gets handled instantly. This is why the bot feels "snappy" even during high traffic.
