//! bpipe-scrape — Standalone statarea.com predictions scraper.
//!
//! Scrapes predictions and saves them to a JSON file.
//! Does NOT require any Betfair credentials.

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use bpipe::scraper;

/// bpipe-scrape — Scrape statarea.com predictions to JSON
#[derive(Parser)]
#[command(
    name = "bpipe-scrape",
    version,
    about = "Scrape football predictions from statarea.com and save to JSON"
)]
struct Cli {
    /// Statarea URL to scrape
    #[arg(
        long,
        default_value = "https://www.statarea.com/predictions"
    )]
    url: String,

    /// Only include upcoming matches (no results yet)
    #[arg(long)]
    upcoming: bool,

    /// Output JSON file path (default: matches_YYYYMMDD_HHMMSS.json)
    #[arg(long, short)]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let output = cli.output.unwrap_or_else(|| {
        let ts = Local::now().format("%Y%m%d_%H%M%S");
        PathBuf::from(format!("matches_{ts}.json"))
    });

    eprintln!("Scraping predictions from {}...", cli.url);
    let mut matches = scraper::scrape_predictions(&cli.url)
        .await
        .context("Failed to scrape predictions")?;

    if cli.upcoming {
        matches.retain(|m| !m.has_result);
    }

    eprintln!("Found {} match(es).", matches.len());

    let json = serde_json::to_string_pretty(&matches)
        .context("Failed to serialize matches to JSON")?;

    std::fs::write(&output, &json)
        .with_context(|| format!("Failed to write {}", output.display()))?;

    eprintln!("Saved to {}", output.display());
    Ok(())
}
