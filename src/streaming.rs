use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_native_tls::TlsConnector;
use tracing::{debug, error, info, warn};

use crate::auth::Session;
use crate::error::{BpipeError, Result};
use crate::types::streaming::*;

/// A streaming client for real-time market and order updates.
pub struct StreamClient {
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<StreamMessage>,
    id_counter: Arc<AtomicI64>,
}

/// Configuration for a streaming connection.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub host: String,
    pub port: u16,
    /// Conflation interval in milliseconds (0 = no conflation).
    pub conflate_ms: Option<i64>,
    /// Heartbeat interval in milliseconds.
    pub heartbeat_ms: Option<i64>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            host: "stream-api.betfair.com".into(),
            port: 443,
            conflate_ms: None,
            heartbeat_ms: Some(5000),
        }
    }
}

impl StreamClient {
    /// Connect to the Betfair Streaming API and authenticate.
    pub async fn connect(session: &Session, config: &StreamConfig) -> Result<Self> {
        let addr = format!("{}:{}", config.host, config.port);
        info!("Connecting to streaming API at {}", addr);

        // Establish TCP + TLS connection
        let tcp_stream = TcpStream::connect(&addr).await?;
        let connector = native_tls::TlsConnector::new()?;
        let connector = TlsConnector::from(connector);
        let tls_stream = connector.connect(&config.host, tcp_stream).await?;

        let (reader, mut writer) = tokio::io::split(tls_stream);
        let mut buf_reader = BufReader::new(reader);

        // Channel for outgoing messages (user → stream)
        let (out_tx, mut out_rx) = mpsc::channel::<String>(64);
        // Channel for incoming messages (stream → user)
        let (in_tx, in_rx) = mpsc::channel::<StreamMessage>(256);

        let id_counter = Arc::new(AtomicI64::new(1));

        // Read the initial connection message
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        debug!("Stream connection message: {}", line.trim());

        let conn_msg: StreamMessage = serde_json::from_str(line.trim()).map_err(|e| {
            BpipeError::StreamError(format!("Failed to parse connection message: {}", e))
        })?;

        match &conn_msg {
            StreamMessage::Connection { connection_id } => {
                info!("Stream connected: connectionId={}", connection_id);
            }
            _ => {
                return Err(BpipeError::StreamError(
                    "Expected connection message".into(),
                ));
            }
        }

        // Send authentication
        let auth_id = id_counter.fetch_add(1, Ordering::SeqCst);
        let auth_msg = serde_json::to_string(&StreamAuthentication {
            op: "authentication".into(),
            id: auth_id,
            app_key: session.app_key.clone(),
            session: session.session_token.clone(),
        })?;
        writer.write_all(auth_msg.as_bytes()).await?;
        writer.write_all(b"\r\n").await?;
        writer.flush().await?;
        debug!("Sent authentication message");

        // Read auth response
        line.clear();
        buf_reader.read_line(&mut line).await?;
        debug!("Stream auth response: {}", line.trim());

        let auth_resp: StreamMessage = serde_json::from_str(line.trim()).map_err(|e| {
            BpipeError::StreamError(format!("Failed to parse auth response: {}", e))
        })?;

        match &auth_resp {
            StreamMessage::Status {
                status_code,
                error_code,
                error_message,
                ..
            } => {
                let code = status_code.as_deref().unwrap_or("UNKNOWN");
                if code != "SUCCESS" {
                    return Err(BpipeError::StreamError(format!(
                        "Stream auth failed: code={}, error={:?}, message={:?}",
                        code, error_code, error_message
                    )));
                }
                info!("Stream authenticated successfully");
            }
            _ => {
                return Err(BpipeError::StreamError(
                    "Expected status message after auth".into(),
                ));
            }
        }

        // Spawn reader task
        let in_tx_clone = in_tx.clone();
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        info!("Stream connection closed by server");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        // Log raw message (truncated) so we can diagnose issues
                        let preview = if trimmed.len() > 500 {
                            format!("{}...[{} bytes total]", &trimmed[..500], trimmed.len())
                        } else {
                            trimmed.to_string()
                        };
                        debug!("RAW stream msg: {}", preview);
                        match serde_json::from_str::<StreamMessage>(trimmed) {
                            Ok(msg) => {
                                if in_tx_clone.send(msg).await.is_err() {
                                    debug!("Stream receiver dropped, stopping reader");
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse stream message: {} — raw: {}", e, preview);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream read error: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn writer task
        tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                if let Err(e) = writer.write_all(msg.as_bytes()).await {
                    error!("Stream write error: {}", e);
                    break;
                }
                if let Err(e) = writer.write_all(b"\r\n").await {
                    error!("Stream write error: {}", e);
                    break;
                }
                if let Err(e) = writer.flush().await {
                    error!("Stream flush error: {}", e);
                    break;
                }
            }
        });

        Ok(Self {
            sender: out_tx,
            receiver: in_rx,
            id_counter,
        })
    }

    /// Subscribe to market changes for the given market IDs.
    pub async fn subscribe_markets(
        &self,
        market_ids: Vec<String>,
        fields: Option<Vec<String>>,
        ladder_levels: Option<i32>,
    ) -> Result<i64> {
        self.subscribe_markets_with_config(market_ids, fields, ladder_levels, None, None).await
    }

    /// Subscribe to market changes with explicit heartbeat and conflation settings.
    pub async fn subscribe_markets_with_config(
        &self,
        market_ids: Vec<String>,
        fields: Option<Vec<String>>,
        ladder_levels: Option<i32>,
        heartbeat_ms: Option<i64>,
        conflate_ms: Option<i64>,
    ) -> Result<i64> {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst);

        let data_filter = Some(StreamMarketDataFilter {
            ladder_levels,
            fields: Some(fields.unwrap_or_else(|| {
                vec![
                    "EX_BEST_OFFERS".into(),
                    "EX_TRADED".into(),
                    "EX_LTP".into(),
                    "EX_MARKET_DEF".into(),
                ]
            })),
        });

        let sub = MarketSubscription {
            op: "marketSubscription".into(),
            id,
            market_filter: StreamMarketFilter {
                market_ids: Some(market_ids),
                ..Default::default()
            },
            market_data_filter: data_filter,
            initial_clk: None,
            clk: None,
            conflate_ms,
            heartbeat_ms,
        };

        let msg = serde_json::to_string(&sub)?;
        info!("Sending subscription: {}", msg);
        self.sender
            .send(msg)
            .await
            .map_err(|e| BpipeError::StreamError(format!("Failed to send subscription: {}", e)))?;

        info!("Subscribed to markets (id={})", id);
        Ok(id)
    }

    /// Subscribe to order changes.
    pub async fn subscribe_orders(&self) -> Result<i64> {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst);

        let sub = OrderSubscription {
            op: "orderSubscription".into(),
            id,
            order_filter: Some(OrderSubscriptionFilter::default()),
            initial_clk: None,
            clk: None,
            conflate_ms: None,
            heartbeat_ms: None,
        };

        let msg = serde_json::to_string(&sub)?;
        self.sender
            .send(msg)
            .await
            .map_err(|e| BpipeError::StreamError(format!("Failed to send order subscription: {}", e)))?;

        info!("Subscribed to orders (id={})", id);
        Ok(id)
    }

    /// Send a heartbeat.
    pub async fn heartbeat(&self) -> Result<()> {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst);
        let hb = StreamHeartbeat {
            op: "heartbeat".into(),
            id,
        };
        let msg = serde_json::to_string(&hb)?;
        self.sender
            .send(msg)
            .await
            .map_err(|e| BpipeError::StreamError(format!("Failed to send heartbeat: {}", e)))?;
        Ok(())
    }

    /// Receive the next stream message. Returns None if the stream is closed.
    pub async fn next_message(&mut self) -> Option<StreamMessage> {
        self.receiver.recv().await
    }
}
