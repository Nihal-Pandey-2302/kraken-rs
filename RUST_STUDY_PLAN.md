# 🎓 Rust Masterclass: The 2-Week Architecture Study Plan

You want to understand _exactly_ how your Kraken SDK works deep under the hood. This plan maps the complex topics in your code to the specific chapters in "The Rust Book" (TRPL) and provides "Mini-Labs" to prove you know it.

---

## 📅 Week 1: The Foundation (Data & Sync)

### 🗓️ Days 1-2: Serde & The Shape of Data (`src/models.rs`)

Your SDK deals with messy JSON from Kraken. We use `Serde` to tame it.

- **Concept**: Serialization, Enums, Zero-Copy Parsing.
- **The "Why"**: Kraken sends data as `[123, "500.0"]` (mixed types). Standard Rust structs can't handle this without magic.
- **📚 Read**:
  - [TRPL Ch 6: Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
  - [TRPL Ch 10: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
  - [Serde Docs: Untagged Enums](https://serde.rs/enum-representations.html#untagged) (Crucial!)
- **🧪 Mini-Lab**: `lab_serde.rs`
  - Create a struct that assumes input is `{"foo": 1}`.
  - Now make it accept `{"foo": "1"}` (string) OR `{"foo": 1}` (int) using `#[serde(untagged)]`.
  - Parse `["price", "volume"]` into a valid Rust struct.

### 🗓️ Days 3-4: Concurrency vs Parallelism (`src/lib.rs`)

Your SDK does 5 things at once. How?

- **Concept**: Async/Await, Futures, The Tokio Runtime.
- **The "Why"**: We can't block the CPU waiting for a network packet. We need to "yield" execution.
- **📚 Read**:
  - [TRPL Ch 16: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
  - [The Async Book: 1. Getting Started](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
  - [The Async Book: 2. Under the Hood](https://rust-lang.github.io/async-book/02_execution/01_chapter.html) (How `poll` works)
- **🧪 Mini-Lab**: `lab_async.rs`
  - Write a function `async fn slow_add(a: i32, b: i32) -> i32` that sleeps for 2 seconds.
  - Call it twice sequentially (Total 4s).
  - Call it twice using `tokio::join!` (Total 2s). **Prove** the speedup.

### 🗓️ Day 5: Channels (The Nervous System)

- **Concept**: `mpsc` vs `broadcast`.
- **The "Why"**: The websocket loop updates the OrderBook. The User wants to know. They live in different threads. They can't share memory safely without locks. Channels solve this.
- **📚 Read**:
  - [TRPL Ch 16.2: Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
  - [Tokio Docs: Channels](https://tokio.rs/tokio/tutorial/channels)
- **🧪 Mini-Lab**: `lab_channels.rs`
  - Spawn a "Manager" task.
  - Spawn 3 "Worker" tasks.
  - Have Workers send "Done" to the Manager.
  - Have Manager print "All Done" only when all 3 arrive.

---

## 📅 Week 2: The Architecture (The "Grandmaster" Level)

### 🗓️ Days 6-7: The `select!` Macro (`src/events.rs`)

- **Concept**: Multiplexing futures.
- **The "Why"**: Be able to stop the bot (Control-C) _instantly_ even if it's waiting for a price update.
- **📚 Read**:
  - [Tokio Docs: Select](https://tokio.rs/tokio/tutorial/select)
- **🧪 Mini-Lab**: `lab_select.rs`
  - Create two async functions: one sleeps 1s, one sleeps 5s.
  - Race them in a `select!` loop.
  - Make sure the program exits as soon as the 1s timer wins.

### 🗓️ Days 8-10: The Actor Pattern (The Glue)

- **Concept**: Managing state inside a loop.
- **The "Why"**: To make the WebSocket client "resilient" (auto-reconnect) without the user managing it manually.
- **📚 Read**:
  - [Alice Ryhl: Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/) (The Bible of Rust Actors)
- **🧪 Mini-Lab**: `lab_actor.rs`
  - Build a "Counter" Actor.
  - It owns a `count: i32` integer.
  - The Main thread has _no access_ to `count`.
  - Main sends `Command::Increment` via channel.
  - Actor prints the new number.

---

## 🛠️ How to Practice

1.  Create a `scratchpad` folder in your project (`mkdir scratchpad`).
2.  Add `[[bin]]` entries to `Cargo.toml` for each lab so you can run them (`cargo run --bin lab_async`).
3.  **Try to break it.** What happens if you drop the receiver? What happens if you verify the wrong checksum?

Ready to begin Week 1? 🚀
