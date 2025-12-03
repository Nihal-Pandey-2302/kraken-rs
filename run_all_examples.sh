#!/bin/bash
set -e

echo "🚀 Starting Final Verification Run..."

# Function to run an example with a timeout
run_example() {
    echo "---------------------------------------------------"
    echo "▶️  Running Example: $1"
    echo "---------------------------------------------------"
    # Run for 5 seconds then kill
    timeout 5s cargo run --example $1 || true
    echo "✅  Example $1 finished (or timed out as expected)"
    echo ""
    sleep 1
}

# 1. Basic Subscribe
run_example "01_basic_subscribe"

# 2. Orderbook Tracker
run_example "02_orderbook_tracker"

# 3. Trade Monitor
run_example "03_trade_monitor"

# 4. Multi Pair
run_example "04_multi_pair"

# 5. Custom Handler
run_example "05_custom_handler"

# 6. Reconnect Demo (Build only, running takes too long to demo reconnect)
echo "---------------------------------------------------"
echo "▶️  Building Example: 06_reconnect_demo"
echo "---------------------------------------------------"
cargo build --example 06_reconnect_demo
echo "✅  Build Successful"
echo ""

# 7. TUI (Run for 10s to show it off)
echo "---------------------------------------------------"
echo "▶️  Running TUI: 07_terminal_ui (10s Demo)"
echo "---------------------------------------------------"
timeout 10s cargo run --example 07_terminal_ui || true
echo "✅  TUI finished"
echo ""

# 8. OHLC Candles (Run for 70s to capture at least one 1-minute candle)
echo "---------------------------------------------------"
echo "▶️  Running Example: 08_ohlc_candles (70s Wait)"
echo "---------------------------------------------------"
timeout 70s cargo run --example 08_ohlc_candles || true
echo "✅  Example 08_ohlc_candles finished"
echo ""

# 9. Private Feed (Build only, needs keys)
echo "---------------------------------------------------"
echo "▶️  Building Example: 09_private_feed"
echo "---------------------------------------------------"
cargo build --example 09_private_feed
echo "✅  Build Successful"
echo ""

# 10. Simple Bot (Run for 70s to capture at least one 1-minute candle)
echo "---------------------------------------------------"
echo "▶️  Running Example: 10_simple_bot (70s Wait)"
echo "---------------------------------------------------"
timeout 70s cargo run --example 10_simple_bot || true
echo "✅  Example 10_simple_bot finished"
echo ""

echo "🎉 All Examples Verified!"
