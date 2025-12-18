# 📚 Domain Knowledge: The theory Behind the Code

You've built a **High-Performance Market Data Connector**. To explain _why_ this matters, you need to understand the financial ecosystem it lives in.

Here are the 4 Pillars of Domain Knowledge you need for the interview.

---

## 🏛️ Topic 1: Market Structure (CEX vs DEX)

### The Context

Kraken is a **Centralized Exchange (CEX)**. Unlike a DEX (Uniswap) which lives on-chain, a CEX lives on high-speed servers (usually AWS or bare metal).

- **Why it matters**: CEXs process millions of orders per second. They are the "Source of Truth" for price discovery for major assets like Bitcoin.
- **The Problem**: Because they are so fast, getting data from them via standard HTTP requests (Polling) is too slow. By the time you get the price, it has changed.
- **Your Solution**: Your SDK maintains a constant "pipe" (WebSocket) to receive updates the _microsecond_ they happen.

### 📖 Resource to Study

- **Read**: ["Centralized vs Decentralized Exchanges" (Binance Academy)](https://academy.binance.com/en/articles/what-are-centralized-exchanges-cex)

---

## ⚡ Topic 2: WebSockets vs REST (Real-Time Data)

### The Theory

- **REST (HTTP)**: "Request -> Response". Like mailing a letter and waiting for a reply. High latency. Good for "What is my balance?".
- **WebSockets**: "Open Connection". Like a phone call. The server pushes data to you instantly.
- **Why we need it**: In trading, speed is alpha.
  - REST Latency: ~300ms (TCP Handshake + Request + Response).
  - WebSocket Latency: ~50ms (Data pushed directly over established connection).

### 🔍 Real Life Implementation

Every major financial dashboard (Bloomberg Terminal, TradingView) and every High-Frequency Trading (HFT) bot uses WebSockets. Your SDK is the building block for _any_ of these.

### 📖 Resource to Study

- **Read**: ["WebSockets - A Conceptual Deep Dive" (Abylay Keldibek on Medium)](https://medium.com/@abylay.keldibek/websockets-101-a-conceptual-deep-dive-427529a6e608)

---

## 📉 Topic 3: The Order Book (The Heart of the Market)

### The Concept

The "Price" of Bitcoin isn't a single number. It is a **Queue** of people willing to buy (Bids) and people willing to sell (Asks).

- **The Spread**: The gap between the highest Bid and lowest Ask.
- **Depth**: How much volume is available at each price level.
- **Matching Engine**: The CEX's server which pairs a Buy with a Sell.

### What Your Code Does (`models.rs`)

Your SDK doesn't just get "Price". It syncs the **Entire Book**.

- **Snapshot**: "Here is the state of the book right now."
- **Delta Update**: "Someone just removed 0.5 BTC from the bid at $90,000."
- **Checksum**: You calculate a CRC32 checksum to prove your local book matches the exchange's book exactly. This is critical for HFT firms; if your book is wrong, you lose money.

### 📖 Resource to Study

- **Read**: ["How does an Order Book work?" (Investopedia)](https://www.investopedia.com/terms/o/orderbook.asp)
- **Deep Dive**: ["Building a Local Order Book" (Kraken Docs)](https://docs.kraken.com/websockets/#book-checksum)

---

## 🤖 Topic 4: The HFT Pipeline (The "Why")

### The Big Picture

Why would someone download your SDK? To build an **Automated Trading System**.

**The Pipeline**:

1.  **Ingestion** (Your SDK): Get data fast, normalize it to Rust structs.
2.  **Strategy** (The User): "If Price > SMA(20) and Spread < 0.1%, then BUY."
3.  **Execution** (The SDK): Send the order.

Your project handles step #1 and #3 efficiently so the user can focus on #2.

### 📖 Resource to Study

- **Read**: ["Architecture of a High-Frequency Trading System" (QuantStart)](https://www.quantstart.com/articles/High-Frequency-Trading-Architecture-Part-I/)
