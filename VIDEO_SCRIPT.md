# 🎥 Kraken Forge Hacker Demo: Video Script (3-5 Minutes)

**Target Length:** ~4:00 minutes
**Tone:** Confident, Technical, Fast-paced.
**Allowed Windows:** GitHub, VS Code, Terminal, Docs Site.

---

## 🎬 Scene 1: The Problem (0:00 - 0:45)

**Visual: [Window: Browser - GitHub Repo]**

- Start at the top of `README.md`.
- Scroll down to the **"Clear Problem Statement"** section.
- Highlight the text about "Fragile Data Ingestion" with your mouse.

**Audio (Voiceover):**

> "Building high-performance trading systems on Kraken starts with a challenge: The data firehose.
> Developers spend weeks handling raw WebSocket connections, parsing complex mixed-type JSON arrays, and fighting race conditions.
> One change in the API, or one network blip, and your bot crashes.
> I wanted to fix this. Investing in infrastructure shouldn't mean reinventing the wheel."

---

## 🎬 Scene 2: The Solution (0:45 - 1:30)

**Visual: [Window: VS Code]**

- Open `src/models.rs`.
- Scroll slowly through the structs. Highlight `#[derive(Deserialize)]` lines.
- _Optional_: Quickly switch to `src/lib.rs` to show the `EventLoop`.

**Audio (Voiceover):**

> "Introducing the Kraken Rust SDK.
> I built a production-ready, asynchronous client that turns chaos into order.
> Under the hood, it uses an Actor-based architecture to manage state seamlessly.
> We use strict Rust types and `serde` with custom zero-copy deserializers to handle Kraken's unique data format.
> This ensures that if it compiles, it works. No runtime surprises."

---

## 🎬 Scene 3: The "Grandmaster" TUI (1:30 - 2:45) **[HERO MOMENT]**

**Visual: [Window: Terminal]**

- Run: `docker compose run --rm --build kraken-tui`
- **Action**: The TUI loads instantly.
- **Action**: Switch between tabs using `1` (Market) and `2` (Analytics).
- **Action**: Point out the "Sparklines" and "Liquidity Meter".

**Audio (Voiceover):**

> "But a backend SDK is hard to visualize. So I built this: The Grandmaster Terminal UI.
> Running entirely on the SDK, this dashboard processes over 600,000 messages per second.
> On the Market Tab, you see valid, checksum-verified order books.
> On the Analytics Tab, we have real-time candle aggregation and simple moving averages calculated on the fly.
> This isn't just a demo—it's a stress test. The UI remains responsive at 60fps even during high volatility."

---

## 🎬 Scene 4: Algorithmic Trading (2:45 - 3:15)

**Visual: [Window: VS Code]**

- Open `examples/10_simple_bot.rs`.
- Highlight the logic: `if fast_sma > slow_sma { BUY }`.

**Visual: [Window: Terminal]**

- Run: `cargo run --example 10_simple_bot`
- Show the logs printing "Building History..."

**Audio (Voiceover):**

> "The SDK enables sophisticated strategies with minimal code.
> Here is a complete SMA Crossover bot in under 100 lines of Rust.
> It handles the connection, subscriptions, and even candle generation automatically.
> You just focus on the Alpha."

---

## 🎬 Scene 5: Reliability & Docs (3:15 - 3:45)

**Visual: [Window: Terminal]**

- Run: `./run_all_examples.sh`.
- Let the green checks scroll by.

**Visual: [Window: Browser - Documentation Site]**

- Show your GitHub Pages Docs (`https://nihal-pandey-2302.github.io/kraken-rs/`).
- Click on `KrakenClient` struct to show the detailed API reference.

**Audio (Voiceover):**

> "Speed means nothing without correctness.
> This SDK implements Kraken's CRC32 checksum validation algorithm to guarantee data integrity.
> We also include a comprehensive test suite and full API documentation generated directly from the source code."

---

## 🎬 Scene 6: Conclusion (3:45 - 4:00)

**Visual: [Window: Browser - GitHub Repo]**

- Show the top of the repo again. Focus on the "Hackathon Submission" and "Documentation" badges.

**Audio (Voiceover):**

> "The Kraken Rust SDK is open source, dockerized, and ready for production.
> It bridges the gap between raw API power and developer experience.
> Thank you."

---

## 🎥 Recording Checklist

1.  [ ] **Open these 4 windows** before starting:
    - Browser Tab 1: GitHub Repo
    - Browser Tab 2: Docs Site
    - VS Code (Clean view, `src/models.rs` open)
    - Terminal (Large font, cleared)
2.  [ ] **Switching**: Use `Alt+Tab` or swipe between full-screen spaces for smooth transitions.
