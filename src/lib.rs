//! # bpipe — Betfair Exchange API Client
//!
//! A Rust library and CLI for interacting with the Betfair Exchange API in real time.
//!
//! ## Features
//! - **Authentication**: Interactive login (username/password) and certificate-based bot login
//! - **Betting API**: Browse sports, events, markets; place/cancel/replace bets
//! - **Account API**: Check balance, account details
//! - **Streaming API**: Real-time market price and order change subscriptions via TCP/TLS
//!
//! ## Quick Start (library)
//! ```rust,no_run
//! use bpipe::{BetfairConfig, auth, BetfairClient};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = BetfairConfig::interactive("YOUR_APP_KEY", "username", "password");
//!     let session = auth::login(&config).await?;
//!     let client = BetfairClient::new(session, &config);
//!
//!     let funds = client.get_account_funds().await?;
//!     println!("Balance: {:?}", funds.available_to_bet_balance);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod types;
pub mod betting;
pub mod account;
pub mod streaming;
pub mod orderbook;
pub mod scraper;

// Re-exports for convenience
pub use client::BetfairClient;
pub use config::{AuthConfig, BetfairConfig, ApiUrls};
pub use error::{BpipeError, Result};
pub use streaming::{StreamClient, StreamConfig};
pub use orderbook::OrderBook;

