use eyre::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use tracing::{info, error, warn};
use tokio::sync::{broadcast, mpsc};
use serde::Serialize;

pub mod models;
pub mod aggregator;
pub mod auth;
use models::KrakenEvent;

#[derive(Debug, Clone)]
pub enum Command {
    Subscribe {
        pairs: Vec<String>,
        subscription: SubscriptionArgs,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionArgs {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

pub struct KrakenClient {
    ws_url: String,
    event_sender: broadcast::Sender<KrakenEvent>,
    command_sender: mpsc::Sender<Command>,
    // We store the receiver in an Option so we can take it out once when connecting
    command_receiver: std::sync::Mutex<Option<mpsc::Receiver<Command>>>, 
}

impl KrakenClient {
    /// Creates a new `KrakenClient` instance.
    ///
    /// This initializes the internal channels but does not connect to the WebSocket yet.
    /// Call `connect()` to establish the connection.
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        let (command_sender, command_receiver) = mpsc::channel(100);
        Self {
            ws_url: "wss://ws.kraken.com".to_string(),
            event_sender,
            command_sender,
            command_receiver: std::sync::Mutex::new(Some(command_receiver)),
        }
    }

    /// Returns a broadcast receiver for Kraken events.
    ///
    /// You can call this multiple times to create multiple subscribers (e.g., one for logging, one for trading).
    pub fn subscribe_events(&self) -> broadcast::Receiver<KrakenEvent> {
        self.event_sender.subscribe()
    }

    /// Subscribes to a list of pairs on a specific channel.
    ///
    /// # Arguments
    ///
    /// * `pairs` - A list of trading pairs (e.g., `vec!["XBT/USD".to_string()]`).
    /// * `name` - The channel name (e.g., `"trade"`, `"book"`, `"ticker"`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use kraken_sdk::KrakenClient;
    /// # async fn example() {
    /// let client = KrakenClient::new();
    /// client.subscribe(vec!["XBT/USD".to_string()], "trade", None).await.unwrap();
    /// # }
    /// ```
    pub async fn subscribe(&self, pairs: Vec<String>, name: &str, token: Option<String>) -> Result<()> {
        let cmd = Command::Subscribe {
            pairs,
            subscription: SubscriptionArgs { 
                name: name.to_string(),
                token,
            },
        };
        self.command_sender.send(cmd).await.map_err(|e| eyre::eyre!("Failed to send command: {}", e))?;
        Ok(())
    }

    /// Connects to the Kraken WebSocket API and starts the event loop.
    ///
    /// This spawns a background task that handles:
    /// - WebSocket connection and reconnection.
    /// - Parsing incoming messages.
    /// - Sending outgoing commands.
    /// - Broadcasting events to subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the client has already connected (the command receiver is taken).
    pub async fn connect(&self) -> Result<()> {
        info!("Starting Kraken Client...");
        
        // Take the command receiver
        let mut command_receiver = self.command_receiver.lock().unwrap().take()
            .ok_or_else(|| eyre::eyre!("Client already connected (receiver taken)"))?;

        let ws_url = self.ws_url.clone();
        let event_sender = self.event_sender.clone();
        
        // State to track active subscriptions for re-subscribing
        // We use a simple list of commands that we've sent.
        // In a real app, we might want to be smarter (e.g. remove unsubscribes), 
        // but for now, replaying the "Subscribe" commands is sufficient.
        let mut active_subscriptions: Vec<Command> = Vec::new();

        // Spawn the driver task
        tokio::spawn(async move {
            loop {
                info!("Connecting to {}...", ws_url);
                let ws_stream = match connect_async(&ws_url).await {
                    Ok((stream, _)) => {
                        info!("Connected to Kraken WebSocket API");
                        stream
                    }
                    Err(e) => {
                        error!("Connection failed: {}. Retrying in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let (mut write, mut read) = ws_stream.split();

                // Re-send active subscriptions
                for cmd in &active_subscriptions {
                    let Command::Subscribe { pairs, subscription } = cmd;
                    let msg = serde_json::json!({
                        "event": "subscribe",
                        "pair": pairs,
                        "subscription": subscription
                    });
                    if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                            error!("Failed to resubscribe: {}", e);
                            // If we can't send, the connection is likely dead, break to outer loop
                            break; 
                    }
                    info!("Resubscribed to {:?}", pairs);
                }

                loop {
                    tokio::select! {
                        // 1. Handle incoming WS messages
                        msg_opt = read.next() => {
                            match msg_opt {
                                Some(Ok(Message::Text(text))) => {
                                    match serde_json::from_str::<KrakenEvent>(&text) {
                                        Ok(event) => {
                                            let _ = event_sender.send(event);
                                        }
                                        Err(e) => error!("Parse error: {}", e),
                                    }
                                }
                                Some(Ok(Message::Ping(_))) => {}
                                Some(Err(e)) => {
                                    error!("WS Error: {}. Reconnecting...", e);
                                    break; // Break inner loop to reconnect
                                }
                                None => {
                                    warn!("WS Stream ended. Reconnecting...");
                                    break; // Break inner loop to reconnect
                                }
                                _ => {}
                            }
                        }
                        // 2. Handle outgoing commands
                        cmd_opt = command_receiver.recv() => {
                            match cmd_opt {
                                Some(cmd) => {
                                    match &cmd {
                                        Command::Subscribe { pairs, subscription } => {
                                            let msg = serde_json::json!({
                                                "event": "subscribe",
                                                "pair": pairs,
                                                "subscription": subscription
                                            });
                                            if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                                                error!("Failed to send subscription: {}", e);
                                                break; // Connection likely dead
                                            }
                                            info!("Sent subscription for {:?}", pairs);
                                            
                                            // Add to active subscriptions
                                            active_subscriptions.push(cmd);
                                        }
                                    }
                                }
                                None => {
                                    warn!("Command channel closed. Shutting down client.");
                                    return; // Exit the task entirely
                                }
                            }
                        }
                    }
                }
                
                // If we broke the inner loop, wait a bit before reconnecting
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }
}
