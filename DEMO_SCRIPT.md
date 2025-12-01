# 🎥 Kraken SDK - Demo Video Script

**Goal**: Create a 2-minute video that wins the "SDK Client" track.
**Key Message**: "High Performance, Type Safety, and Developer Experience."

---

## 🎬 Scene 1: The Hook (0:00 - 0:30)

**Visual**: Terminal with `docker-compose run --rm kraken-tui` running.
**Action**:

1.  Start with a black screen.
2.  Type: `docker-compose run --rm kraken-tui`
3.  **BOOM**: The TUI loads. Green/Red numbers flashing.
4.  **Voiceover**: "This is the Kraken SDK for Rust. Built for high-frequency trading, it handles thousands of updates per second with zero lag."
5.  Show the "Spread" calculation and the "Volume Bars" in the orderbook.

## 🎬 Scene 2: The Performance (0:30 - 1:00)

**Visual**: Split screen (VS Code / Terminal).
**Action**:

1.  Open `examples/benchmark.rs`.
2.  **Voiceover**: "We didn't just wrap the API. We optimized it."
3.  Run: `cargo run --release --example benchmark`
4.  **Visual**: Show the output: `🚀 Throughput: 648,000 msgs/sec`.
5.  **Voiceover**: "It processes over 600,000 messages per second with full type safety. That's 8% faster than raw Python, but with the safety of Rust."

## 🎬 Scene 3: The Developer Experience (1:00 - 1:30)

**Visual**: VS Code showing `src/models.rs`.
**Action**:

1.  Scroll through the `KrakenEvent` enum.
2.  **Voiceover**: "No more parsing JSON manually. The SDK provides strictly typed data models for every Kraken event."
3.  Show `examples/01_basic_subscribe.rs`.
4.  **Voiceover**: "Connecting is as simple as three lines of code."

## 🎬 Scene 4: The Outro (1:30 - 2:00)

**Visual**: GitHub Repo / README.
**Action**:

1.  Show the README with the Architecture diagram.
2.  **Voiceover**: "Open source, MIT licensed, and ready for production. This is Kraken SDK."
3.  **End Screen**: "Vote for us on TAIKAI!"

---

## 🛠️ Prep Checklist

- [ ] Font Size: 18px+ (Make it readable!)
- [ ] Clean Desktop (No clutter)
- [ ] Run `cargo build --release --example benchmark` _before_ recording so it runs instantly.
