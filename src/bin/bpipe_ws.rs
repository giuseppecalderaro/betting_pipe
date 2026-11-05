use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use bpipe::orderbook::OrderBook;
use bpipe::types::streaming::StreamMessage;
use bpipe::{auth, BetfairConfig, StreamClient, StreamConfig};

/// bpipe-ws — WebSocket relay for Betfair market streaming
#[derive(Parser)]
#[command(name = "bpipe-ws", version, about = "WebSocket server relaying live Betfair market data")]
struct Cli {
    /// Betfair Application Key
    #[arg(long, env = "BETFAIR_APP_KEY", hide_env_values = true)]
    app_key: String,

    /// Betfair username
    #[arg(long, env = "BETFAIR_USERNAME", hide_env_values = true)]
    username: String,

    /// Betfair password
    #[arg(long, env = "BETFAIR_PASSWORD", hide_env_values = true)]
    password: String,

    /// Path to client certificate (PEM) for cert-based auth
    #[arg(long, env = "BETFAIR_CERT_PATH")]
    cert: Option<PathBuf>,

    /// Path to client private key (PEM) for cert-based auth
    #[arg(long, env = "BETFAIR_KEY_PATH")]
    key: Option<PathBuf>,

    /// Market ID to stream (e.g. "1.234567890")
    #[arg(long)]
    market: String,

    /// WebSocket server bind address
    #[arg(long, default_value = "127.0.0.1:9001")]
    bind: String,

    /// Ladder levels (price depth)
    #[arg(long, default_value = "10")]
    depth: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Build config
    let config = if let (Some(cert), Some(key)) = (&cli.cert, &cli.key) {
        BetfairConfig::certificate(&cli.app_key, &cli.username, &cli.password, cert, key)
    } else {
        BetfairConfig::interactive(&cli.app_key, &cli.username, &cli.password)
    };

    // Authenticate
    info!("Authenticating with Betfair...");
    let session = auth::login(&config)
        .await
        .context("Authentication failed")?;
    info!("Authenticated successfully");

    // Shared order book state
    let order_book = Arc::new(RwLock::new(OrderBook::new(cli.market.clone())));

    // Broadcast channel: each WebSocket client gets a receiver
    let (tx, _) = broadcast::channel::<String>(256);

    // Start WebSocket server
    let addr: SocketAddr = cli.bind.parse().context("Invalid bind address")?;
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on ws://{}", addr);

    // Spawn WebSocket acceptor
    let ws_tx = tx.clone();
    let ws_book = order_book.clone();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    info!("New WebSocket connection from {}", peer);
                    let rx = ws_tx.subscribe();
                    let book = ws_book.clone();
                    tokio::spawn(handle_ws_connection(stream, rx, book));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    // Connect to Betfair streaming
    let stream_config = StreamConfig {
        host: config.urls.stream_api.clone(),
        port: config.urls.stream_port,
        conflate_ms: Some(50), // Get updates quickly (50ms conflation)
        heartbeat_ms: Some(5000), // Betfair sends heartbeat every 5s
    };

    info!("Connecting to Betfair streaming API...");
    let mut stream = StreamClient::connect(&session, &stream_config)
        .await
        .context("Failed to connect to streaming API")?;

    info!("Subscribing to market {}...", cli.market);
    stream
        .subscribe_markets_with_config(
            vec![cli.market.clone()],
            Some(vec![
                "EX_BEST_OFFERS".into(),
                "EX_TRADED".into(),
                "EX_LTP".into(),
                "EX_MARKET_DEF".into(),
                "EX_TRADED_VOL".into(),
            ]),
            Some(cli.depth),
            stream_config.heartbeat_ms,
            stream_config.conflate_ms,
        )
        .await?;

    info!("Streaming started. Relaying to WebSocket clients...");

    let mut msg_count: u64 = 0;
    let mut change_count: u64 = 0;

    // Main loop: read stream messages, update order book, broadcast
    while let Some(msg) = stream.next_message().await {
        msg_count += 1;
        match msg {
            StreamMessage::MarketChange(mcm) => {
                let publish_time = mcm.pt;
                let ct = mcm.ct.as_deref().unwrap_or("delta");
                let num_changes = mcm.mc.as_ref().map_or(0, |mc| mc.len());
                let num_runners: usize = mcm
                    .mc
                    .as_ref()
                    .map_or(0, |mc| mc.iter().filter_map(|c| c.rc.as_ref()).map(|rc| rc.len()).sum());

                if ct == "SUB_IMAGE" || ct == "RESUB_DELTA" || num_changes > 0 {
                    info!(
                        "MCM #{}: ct={}, markets={}, runners={}, pt={:?}",
                        msg_count, ct, num_changes, num_runners, publish_time
                    );
                } else {
                    debug!(
                        "MCM #{}: ct={} (heartbeat), pt={:?}",
                        msg_count, ct, publish_time
                    );
                }

                let mut book = order_book.write().await;

                if let Some(changes) = &mcm.mc {
                    for change in changes {
                        book.apply_change(change, publish_time);
                        change_count += 1;
                    }
                } else if let Some(pt) = publish_time {
                    // Heartbeat — update timestamp so clients know connection is alive
                    book.last_update = Some(pt);
                }

                // Always broadcast current state (including on heartbeats)
                // so the webapp can update its liveness indicator
                match serde_json::to_string(&*book) {
                    Ok(json) => {
                        let _ = tx.send(json);
                    }
                    Err(e) => {
                        warn!("Failed to serialize order book: {}", e);
                    }
                }

                if change_count > 0 && change_count % 100 == 0 {
                    info!(
                        "Processed {} changes across {} messages, {} runners",
                        change_count,
                        msg_count,
                        book.runners.len()
                    );
                }
            }
            StreamMessage::Status {
                status_code,
                error_code,
                error_message,
                ..
            } => {
                info!(
                    "Stream status: code={:?} error={:?} msg={:?}",
                    status_code, error_code, error_message
                );
            }
            StreamMessage::Connection { connection_id } => {
                info!("Stream reconnected: {}", connection_id);
            }
            StreamMessage::OrderChange(_) => {
                // We don't subscribe to orders in this server
            }
        }
    }

    warn!("Stream closed unexpectedly after {} messages", msg_count);
    Ok(())
}

async fn handle_ws_connection(
    stream: TcpStream,
    mut rx: broadcast::Receiver<String>,
    order_book: Arc<RwLock<OrderBook>>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send the current order book snapshot immediately
    {
        let book = order_book.read().await;
        let runners = book.runners.len();
        if let Ok(json) = serde_json::to_string(&*book) {
            info!(
                "Sending initial snapshot to client: {} runners, {} bytes",
                runners,
                json.len()
            );
            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                error!("Failed to send initial snapshot: {}", e);
                return;
            }
        }
    }

    // Relay broadcast messages to this WebSocket client
    // Also handle incoming messages (ping/pong, close)
    loop {
        tokio::select! {
            // Incoming from broadcast (Betfair updates)
            result = rx.recv() => {
                match result {
                    Ok(json) => {
                        if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                            debug!("WebSocket send error (client disconnected): {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged, skipped {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Broadcast channel closed");
                        break;
                    }
                }
            }
            // Incoming from WebSocket client
            result = ws_receiver.next() => {
                match result {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(_)) => {
                        // Ignore other messages from client
                    }
                    Some(Err(e)) => {
                        debug!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
