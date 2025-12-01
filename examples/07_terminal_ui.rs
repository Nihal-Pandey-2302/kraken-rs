use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    symbols,
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, Gauge, GraphType, Paragraph, Row, Sparkline,
        Table,
    },
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use kraken_sdk::{
    aggregator::TradeAggregator,
    models::{Candle, LocalOrderBook},
    KrakenClient,
};

struct TradeInfo {
    time: String,
    price: String,
    volume: String,
    side: String, // "b" or "s"
}

struct App {
    local_book: LocalOrderBook,
    trades: Vec<TradeInfo>,
    status: String,
    selected_tab: usize,
    aggregator: TradeAggregator,
    candles: Vec<Candle>,
    // New Fields
    price_history: Vec<u64>,
    msg_count: u64,
    start_time: Instant,
    last_latency: u128,
}

impl App {
    fn new() -> Self {
        Self {
            local_book: LocalOrderBook::new(),
            trades: Vec::new(),
            status: "Initializing...".to_string(),
            selected_tab: 0,
            aggregator: TradeAggregator::new(10), // 10-second candles for demo
            candles: Vec::new(),
            price_history: Vec::new(),
            msg_count: 0,
            start_time: Instant::now(),
            last_latency: 0,
        }
    }

    fn get_spread(&self) -> (f64, f64) {
        // Simple spread calculation
        // Asks are sorted Low -> High (Best ask is first)
        // Bids are sorted High -> Low (Best bid is first)
        // Note: LocalOrderBook stores strings in BTreeMap.
        // We need to find the best ask and best bid.

        // Since BTreeMap sorts strings lexicographically, we need to be careful.
        // However, for this demo, we'll iterate and parse to find true best.
        // Optimization: Cache this or use a better data structure in production.

        let best_ask = self
            .local_book
            .asks
            .keys()
            .filter_map(|p| p.parse::<f64>().ok())
            .fold(f64::MAX, |a, b| a.min(b));

        let best_bid = self
            .local_book
            .bids
            .keys()
            .filter_map(|p| p.parse::<f64>().ok())
            .fold(f64::MIN, |a, b| a.max(b));

        if best_ask == f64::MAX || best_bid == f64::MIN {
            return (0.0, 0.0);
        }

        (
            best_ask - best_bid,
            (best_ask - best_bid) / best_ask * 100.0,
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App state
    let mut app = App::new();

    // Setup Kraken Client
    let client = KrakenClient::new();
    let mut rx = client.subscribe_events();

    // Connect and subscribe
    app.status = "Connecting to Kraken WS...".to_string();
    terminal.draw(|f| ui(f, &app))?; // Draw once to show status

    client.connect().await?;
    client
        .subscribe(vec!["XBT/USD".to_string()], "trade", None)
        .await?;
    client
        .subscribe(vec!["XBT/USD".to_string()], "book", None)
        .await?;
    app.status = "Connected. Streaming XBT/USD...".to_string();

    // TUI Loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('1') => app.selected_tab = 0,
                        KeyCode::Char('2') => app.selected_tab = 1,
                        KeyCode::Char('3') => app.aggregator = TradeAggregator::new(10),
                        KeyCode::Char('4') => app.aggregator = TradeAggregator::new(30),
                        KeyCode::Char('5') => app.aggregator = TradeAggregator::new(60),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Drain events
            loop {
                match rx.try_recv() {
                    Ok(event) => {
                        if let Some(trade) = event.clone().try_into_trade_data() {
                            for t in trade.data {
                                // Update Aggregator
                                let trade_time = t.time.parse::<f64>().unwrap_or(0.0);
                                if let Some(candle) = app.aggregator.check_flush(trade_time) {
                                    app.candles.insert(0, candle);
                                    if app.candles.len() > 50 {
                                        app.candles.pop();
                                    }
                                }
                                app.aggregator.update(&t);

                                // Update Stats
                                app.msg_count += 1;
                                app.last_latency = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis()
                                    - (t.time.parse::<f64>().unwrap_or(0.0) * 1000.0) as u128;

                                // Update Price History (Sparkline)
                                let price = t.price.parse::<f64>().unwrap_or(0.0);
                                app.price_history.push(price as u64);
                                if app.price_history.len() > 100 {
                                    app.price_history.remove(0);
                                }

                                // Whale Alert
                                let volume = t.volume.parse::<f64>().unwrap_or(0.0);
                                let value = price * volume;
                                let whale_emoji = if value > 50000.0 { "🐋 " } else { "" };

                                let info = TradeInfo {
                                    time: t.time,
                                    price: t.price,
                                    volume: t.volume,
                                    side: format!("{}{}", whale_emoji, t.side),
                                };
                                app.trades.insert(0, info);
                                if app.trades.len() > 50 {
                                    app.trades.pop();
                                }
                            }
                        } else if let Some(book) = event.try_into_orderbook_data() {
                            app.local_book.update(&book);
                        }
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                    Err(_) => break,
                }
            }
            last_tick = std::time::Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    // Layout: Header, Main (Split), Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    // --- Header ---
    let (spread, spread_pct) = app.get_spread();
    let spread_text = if spread > 0.0 {
        format!("Spread: {:.1} ({:.2}%)", spread, spread_pct)
    } else {
        "Spread: -".to_string()
    };

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(chunks[0]);

    let title = Paragraph::new("🐙 KRAKEN SDK TERMINAL")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));

    let status_color = if app.status.contains("Connected") {
        Color::Green
    } else {
        Color::Yellow
    };
    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(status_color))
        .block(Block::default().borders(Borders::ALL).title("Status"));

    let spread_widget = Paragraph::new(spread_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::ALL).title("XBT/USD"));

    f.render_widget(title, header_chunks[0]);
    f.render_widget(status, header_chunks[1]);
    f.render_widget(spread_widget, header_chunks[2]);

    // --- Sparkline (Overlay on Header) ---
    // We render it in the middle chunk, below the status text if possible, or just replace status?
    // Let's put it in the "Spread" chunk for now, or create a new row.
    // Actually, let's just put it in the middle chunk (Status) but make it small.
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("Price History")
                .borders(Borders::NONE),
        )
        .data(&app.price_history)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(
        sparkline,
        header_chunks[1].inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );

    // --- Tab Content ---
    match app.selected_tab {
        0 => render_market_tab(f, app, chunks[1]),
        1 => render_analytics_tab(f, app, chunks[1]),
        _ => {}
    }

    // --- Footer ---
    let elapsed = app.start_time.elapsed().as_secs_f64();
    let msg_rate = if elapsed > 0.0 {
        app.msg_count as f64 / elapsed
    } else {
        0.0
    };

    let footer_text = format!(
        "Controls: [q] Quit | [1] Market | [2] Analytics | [3] 10s [4] 30s [5] 60s | Latency: {}ms | Msgs/sec: {:.0}", 
        app.last_latency, msg_rate
    );
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

fn render_market_tab(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Orderbook (Left)
    let book_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    // Liquidity Meter (Top of Orderbook)
    let total_bid_vol: f64 = app
        .local_book
        .bids
        .values()
        .filter_map(|v| v.parse::<f64>().ok())
        .sum();
    let total_ask_vol: f64 = app
        .local_book
        .asks
        .values()
        .filter_map(|v| v.parse::<f64>().ok())
        .sum();
    let total_vol = total_bid_vol + total_ask_vol;
    let bid_ratio = if total_vol > 0.0 {
        total_bid_vol / total_vol
    } else {
        0.5
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("Liquidity Imbalance")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Red))
        .ratio(bid_ratio)
        .label(format!(
            "{:.0}% Bids / {:.0}% Asks",
            bid_ratio * 100.0,
            (1.0 - bid_ratio) * 100.0
        ));

    let book_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(book_chunks[0].union(book_chunks[1])); // Span across both columns

    f.render_widget(gauge, book_layout[0]);

    // Split the bottom part back into two columns
    let inner_book_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(book_layout[1]);

    // ... (Use inner_book_chunks instead of book_chunks for tables)

    // Prepare Bids (Green) - Sorted High to Low
    let mut bids: Vec<(&String, &String)> = app.local_book.bids.iter().collect();
    bids.sort_by(|a, b| {
        let p1 = a.0.parse::<f64>().unwrap_or(0.0);
        let p2 = b.0.parse::<f64>().unwrap_or(0.0);
        p2.partial_cmp(&p1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Prepare Asks (Red) - Sorted Low to High
    let mut asks: Vec<(&String, &String)> = app.local_book.asks.iter().collect();
    asks.sort_by(|a, b| {
        let p1 = a.0.parse::<f64>().unwrap_or(0.0);
        let p2 = b.0.parse::<f64>().unwrap_or(0.0);
        p1.partial_cmp(&p2).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Render Bids
    let bid_rows: Vec<Row> = bids
        .iter()
        .take(25)
        .map(|(p, v)| {
            let vol = v.parse::<f64>().unwrap_or(0.0);
            let bar = create_volume_bar(vol, 10.0, 10); // Assume max vol 10 for bar scaling
            Row::new(vec![
                Cell::from(format!("{}", p)).style(Style::default().fg(Color::Green)),
                Cell::from(v.as_str()),
                Cell::from(bar).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let bids_table = Table::new(
        bid_rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Price", "Vol", "Depth"])
            .style(Style::default().add_modifier(Modifier::UNDERLINED)),
    )
    .block(Block::default().borders(Borders::ALL).title("Bids (Buy)"));

    f.render_widget(bids_table, inner_book_chunks[0]);

    // Render Asks
    let ask_rows: Vec<Row> = asks
        .iter()
        .take(25)
        .map(|(p, v)| {
            let vol = v.parse::<f64>().unwrap_or(0.0);
            let bar = create_volume_bar(vol, 10.0, 10);
            Row::new(vec![
                Cell::from(format!("{}", p)).style(Style::default().fg(Color::Red)),
                Cell::from(v.as_str()),
                Cell::from(bar).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let asks_table = Table::new(
        ask_rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Price", "Vol", "Depth"])
            .style(Style::default().add_modifier(Modifier::UNDERLINED)),
    )
    .block(Block::default().borders(Borders::ALL).title("Asks (Sell)"));

    f.render_widget(asks_table, inner_book_chunks[1]);

    // Trades (Right)
    let trade_rows: Vec<Row> = app
        .trades
        .iter()
        .map(|t| {
            let color = if t.side == "b" {
                Color::Green
            } else {
                Color::Red
            };
            Row::new(vec![
                Cell::from(t.time.as_str()),
                Cell::from(t.price.as_str()).style(Style::default().fg(color)),
                Cell::from(t.volume.as_str()),
            ])
        })
        .collect();

    let trades_table = Table::new(
        trade_rows,
        [
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Time", "Price", "Vol"])
            .style(Style::default().add_modifier(Modifier::UNDERLINED)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Trades"),
    );

    f.render_widget(trades_table, main_chunks[1]);
}

fn render_analytics_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // --- Chart (Top) ---
    let candle_data: Vec<(f64, f64)> = app
        .candles
        .iter()
        .rev()
        .enumerate()
        .map(|(i, c)| (i as f64, c.close))
        .collect();

    let datasets = vec![Dataset::default()
        .name("Price")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&candle_data)];

    // Calculate Y-Axis bounds
    let min_price = app
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, |a, b| a.min(b));
    let max_price = app
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, |a, b| a.max(b));
    let y_min = if min_price == f64::MAX {
        0.0
    } else {
        min_price * 0.9999
    }; // Zoom in
    let y_max = if max_price == f64::MIN {
        100.0
    } else {
        max_price * 1.0001
    };

    let chart = Chart::new(datasets)
        .block(Block::default().title("Price Chart").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, app.candles.len() as f64])
                .labels(vec![Span::raw("Old"), Span::raw("New")]),
        )
        .y_axis(
            Axis::default()
                .title("Price")
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", y_min)),
                    Span::raw(format!("{:.0}", y_max)),
                ]),
        );

    f.render_widget(chart, chunks[0]);

    // --- Table (Bottom) ---

    // Calculate SMA-10
    // Simple moving average of Close price
    let mut sma_values = Vec::new();
    let window = 10;
    for i in 0..app.candles.len() {
        if i + window <= app.candles.len() {
            let sum: f64 = app.candles[i..i + window].iter().map(|c| c.close).sum();
            sma_values.push(sum / window as f64);
        } else {
            sma_values.push(0.0); // Not enough data
        }
    }

    let candle_rows: Vec<Row> = app
        .candles
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let color = if c.close >= c.open {
                Color::Green
            } else {
                Color::Red
            };
            let sma = if sma_values[i] > 0.0 {
                format!("{:.2}", sma_values[i])
            } else {
                "-".to_string()
            };
            let trend = if c.close >= c.open {
                "███"
            } else {
                "███"
            };

            Row::new(vec![
                Cell::from(c.start_time.to_string()),
                Cell::from(format!("{:.2}", c.open)),
                Cell::from(format!("{:.2}", c.high)),
                Cell::from(format!("{:.2}", c.low)),
                Cell::from(format!("{:.2}", c.close)).style(Style::default().fg(color)),
                Cell::from(format!("{:.4}", c.volume)),
                Cell::from(sma).style(Style::default().fg(Color::Yellow)),
                Cell::from(trend).style(Style::default().fg(color)),
            ])
        })
        .collect();

    let table = Table::new(
        candle_rows,
        [
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(5),
        ],
    )
    .header(
        Row::new(vec![
            "Time", "Open", "High", "Low", "Close", "Volume", "SMA-10", "Trend",
        ])
        .style(Style::default().add_modifier(Modifier::UNDERLINED)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("OHLCV Candles"),
    );

    f.render_widget(table, chunks[1]);
}

fn create_volume_bar(volume: f64, max_volume: f64, width: usize) -> String {
    let ratio = (volume / max_volume).min(1.0);
    let filled = (ratio * width as f64).round() as usize;
    let bar: String = std::iter::repeat("█").take(filled).collect();
    format!("{:<width$}", bar, width = width)
}
