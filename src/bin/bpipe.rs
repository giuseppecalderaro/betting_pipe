use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use bpipe::types::betting::*;
use bpipe::types::common::*;
use bpipe::{auth, BetfairClient, BetfairConfig, StreamClient, StreamConfig};
use bpipe::scraper::{self, ScrapedMatch};

/// bpipe — Betfair Exchange CLI
#[derive(Parser)]
#[command(name = "bpipe", version, about = "Real-time Betfair Exchange betting CLI")]
struct Cli {
    /// Betfair Application Key
    #[arg(long, env = "BETFAIR_APP_KEY")]
    app_key: String,

    /// Betfair username
    #[arg(long, env = "BETFAIR_USERNAME")]
    username: String,

    /// Betfair password
    #[arg(long, env = "BETFAIR_PASSWORD")]
    password: String,

    /// Path to client certificate (PEM) for cert-based auth
    #[arg(long, env = "BETFAIR_CERT_PATH")]
    cert: Option<PathBuf>,

    /// Path to client private key (PEM) for cert-based auth
    #[arg(long, env = "BETFAIR_KEY_PATH")]
    key: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available sports (event types)
    Sports,

    /// List events for a sport
    Events {
        /// Event type ID (sport ID, e.g. "1" for Soccer)
        #[arg(long)]
        sport: String,

        /// Only show in-play events
        #[arg(long)]
        inplay: bool,
    },

    /// List markets for an event
    Markets {
        /// Event ID
        #[arg(long)]
        event: String,

        /// Max results
        #[arg(long, default_value = "25")]
        max: i32,
    },

    /// Show live prices for a market
    Prices {
        /// Market ID (e.g. "1.234567890")
        #[arg(long)]
        market: String,

        /// Price depth (back/lay levels)
        #[arg(long, default_value = "3")]
        depth: i32,
    },

    /// Place a bet
    Bet {
        /// Market ID
        #[arg(long)]
        market: String,

        /// Selection (runner) ID
        #[arg(long)]
        selection: i64,

        /// Side: back or lay
        #[arg(long)]
        side: String,

        /// Stake amount
        #[arg(long)]
        stake: f64,

        /// Price (odds)
        #[arg(long)]
        price: f64,

        /// Persistence: lapse, persist, or moc
        #[arg(long, default_value = "lapse")]
        persistence: String,
    },

    /// Cancel a bet
    Cancel {
        /// Market ID
        #[arg(long)]
        market: String,

        /// Bet ID to cancel
        #[arg(long)]
        bet_id: String,

        /// Partial cancellation amount (omit for full cancel)
        #[arg(long)]
        amount: Option<f64>,
    },

    /// List current open orders
    Orders {
        /// Filter by market ID
        #[arg(long)]
        market: Option<String>,
    },

    /// Show account balance and details
    Account,

    /// Stream live market data
    Stream {
        /// Market ID(s) to stream, comma-separated
        #[arg(long, value_delimiter = ',')]
        markets: Vec<String>,

        /// Also subscribe to order changes
        #[arg(long)]
        orders: bool,
    },

    /// Scrape statarea.com predictions and match with Betfair events
    Scrape {
        /// Statarea URL to scrape
        #[arg(long, default_value = "https://www.statarea.com/predictions")]
        url: String,

        /// Only show upcoming matches (no results yet)
        #[arg(long)]
        upcoming: bool,
    },

    /// Read a matches JSON file (from bpipe-scrape) and match against Betfair events
    MatchEvents {
        /// Path to the matches JSON file
        #[arg(long, short, default_value = "matches.json")]
        input: PathBuf,

        /// Only show upcoming matches (no results yet)
        #[arg(long)]
        upcoming: bool,

        /// Output file for the matched table
        #[arg(long, short, default_value = "matched.txt")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
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
    eprintln!("Authenticating...");
    let session = auth::login(&config)
        .await
        .context("Authentication failed")?;
    eprintln!("Authenticated successfully.\n");

    let client = BetfairClient::new(session.clone(), &config);

    match cli.command {
        Command::Sports => cmd_sports(&client).await?,
        Command::Events { sport, inplay } => cmd_events(&client, &sport, inplay).await?,
        Command::Markets { event, max } => cmd_markets(&client, &event, max).await?,
        Command::Prices { market, depth } => cmd_prices(&client, &market, depth).await?,
        Command::Bet {
            market,
            selection,
            side,
            stake,
            price,
            persistence,
        } => cmd_bet(&client, &market, selection, &side, stake, price, &persistence).await?,
        Command::Cancel {
            market,
            bet_id,
            amount,
        } => cmd_cancel(&client, &market, &bet_id, amount).await?,
        Command::Orders { market } => cmd_orders(&client, market.as_deref()).await?,
        Command::Account => cmd_account(&client).await?,
        Command::Stream { markets, orders } => {
            cmd_stream(&session, &config, markets, orders).await?
        }
        Command::Scrape { url, upcoming } => cmd_scrape(&client, &url, upcoming).await?,
        Command::MatchEvents { input, upcoming, output } => cmd_match_events(&client, &input, upcoming, &output).await?,
    }

    Ok(())
}

// ─── Command implementations ────────────────────────────

async fn cmd_sports(client: &BetfairClient) -> Result<()> {
    let filter = MarketFilter::default();
    let results = client.list_event_types(&filter).await?;

    println!("{:<8} {:<30} {:>10}", "ID", "Sport", "Markets");
    println!("{}", "-".repeat(50));
    for r in &results {
        println!(
            "{:<8} {:<30} {:>10}",
            r.event_type.id,
            r.event_type.name,
            r.market_count.unwrap_or(0)
        );
    }
    println!("\n{} sport(s) found", results.len());
    Ok(())
}

async fn cmd_events(client: &BetfairClient, sport_id: &str, inplay: bool) -> Result<()> {
    let filter = MarketFilter {
        event_type_ids: Some(vec![sport_id.to_string()]),
        in_play_only: if inplay { Some(true) } else { None },
        ..Default::default()
    };
    let results = client.list_events(&filter).await?;

    println!(
        "{:<12} {:<50} {:<6} {:>10}",
        "ID", "Event", "CC", "Markets"
    );
    println!("{}", "-".repeat(80));
    for r in &results {
        println!(
            "{:<12} {:<50} {:<6} {:>10}",
            r.event.id,
            truncate(&r.event.name, 48),
            r.event.country_code.as_deref().unwrap_or(""),
            r.market_count.unwrap_or(0)
        );
    }
    println!("\n{} event(s) found", results.len());
    Ok(())
}

async fn cmd_markets(client: &BetfairClient, event_id: &str, max: i32) -> Result<()> {
    let request = ListMarketCatalogueRequest {
        filter: MarketFilter {
            event_ids: Some(vec![event_id.to_string()]),
            ..Default::default()
        },
        market_projection: Some(vec![
            MarketProjection::MarketStartTime,
            MarketProjection::RunnerDescription,
            MarketProjection::EventType,
            MarketProjection::Event,
            MarketProjection::MarketDescription,
        ]),
        sort: Some(MarketSort::FirstToStart),
        max_results: Some(max),
        locale: None,
    };

    let results = client.list_market_catalogue(&request).await?;

    for m in &results {
        println!("Market: {} — {}", m.market_id, m.market_name);
        if let Some(start) = &m.market_start_time {
            println!("  Start: {}", start);
        }
        if let Some(matched) = m.total_matched {
            println!("  Total matched: £{:.2}", matched);
        }
        if let Some(runners) = &m.runners {
            println!("  Runners:");
            for r in runners {
                println!(
                    "    [{:<10}] {}{}",
                    r.selection_id,
                    r.runner_name,
                    r.handicap
                        .map(|h| format!(" (hc: {})", h))
                        .unwrap_or_default()
                );
            }
        }
        println!();
    }
    println!("{} market(s) found", results.len());
    Ok(())
}

async fn cmd_prices(client: &BetfairClient, market_id: &str, depth: i32) -> Result<()> {
    let request = ListMarketBookRequest {
        market_ids: vec![market_id.to_string()],
        price_projection: Some(PriceProjection {
            price_data: Some(vec![PriceData::ExBestOffers, PriceData::ExTraded]),
            ex_best_offers_overrides: Some(ExBestOffersOverrides {
                best_prices_depth: Some(depth),
                rollup_model: None,
                tabk_size: None,
                rollup_percentage: None,
                rollup_limit: None,
            }),
            virtualise: None,
            rollover_limit: None,
        }),
        order_projection: None,
        match_projection: None,
        currency_code: None,
        locale: None,
    };

    let books = client.list_market_book(&request).await?;

    for book in &books {
        println!("Market: {}", book.market_id);
        println!(
            "  Status: {:?}  In-play: {:?}  Delay: {}s",
            book.status.unwrap_or(MarketStatus::Open),
            book.inplay.unwrap_or(false),
            book.bet_delay.unwrap_or(0)
        );
        if let Some(total) = book.total_matched {
            println!("  Total matched: £{:.2}", total);
        }
        println!();

        if let Some(runners) = &book.runners {
            println!(
                "  {:<12} {:>6} {:>28} {:>28}",
                "Selection", "Status", "BACK (price × size)", "LAY (price × size)"
            );
            println!("  {}", "-".repeat(76));

            for runner in runners {
                let status = runner
                    .status
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();

                let backs = runner
                    .ex
                    .as_ref()
                    .and_then(|ex| ex.available_to_back.as_ref())
                    .map(|prices| {
                        prices
                            .iter()
                            .map(|p| format!("{:.2}×{:.2}", p.price, p.size))
                            .collect::<Vec<_>>()
                            .join("  ")
                    })
                    .unwrap_or_default();

                let lays = runner
                    .ex
                    .as_ref()
                    .and_then(|ex| ex.available_to_lay.as_ref())
                    .map(|prices| {
                        prices
                            .iter()
                            .map(|p| format!("{:.2}×{:.2}", p.price, p.size))
                            .collect::<Vec<_>>()
                            .join("  ")
                    })
                    .unwrap_or_default();

                let ltp = runner
                    .last_price_traded
                    .map(|p| format!(" (LTP: {:.2})", p))
                    .unwrap_or_default();

                println!(
                    "  {:<12} {:>6} {:>28} {:>28}{}",
                    runner.selection_id, status, backs, lays, ltp
                );
            }
        }
        println!();
    }
    Ok(())
}

async fn cmd_bet(
    client: &BetfairClient,
    market_id: &str,
    selection_id: i64,
    side: &str,
    stake: f64,
    price: f64,
    persistence: &str,
) -> Result<()> {
    let side = match side.to_uppercase().as_str() {
        "BACK" | "B" => Side::Back,
        "LAY" | "L" => Side::Lay,
        _ => anyhow::bail!("Side must be 'back' or 'lay'"),
    };

    let persistence_type = match persistence.to_uppercase().as_str() {
        "LAPSE" | "L" => PersistenceType::Lapse,
        "PERSIST" | "P" => PersistenceType::Persist,
        "MOC" | "MARKET_ON_CLOSE" => PersistenceType::MarketOnClose,
        _ => anyhow::bail!("Persistence must be 'lapse', 'persist', or 'moc'"),
    };

    println!("Placing {} bet:", side);
    println!("  Market:    {}", market_id);
    println!("  Selection: {}", selection_id);
    println!("  Stake:     £{:.2}", stake);
    println!("  Price:     {:.2}", price);
    println!("  Persist:   {:?}", persistence_type);
    println!();

    let request = PlaceOrdersRequest {
        market_id: market_id.to_string(),
        instructions: vec![PlaceInstruction {
            order_type: OrderType::Limit,
            selection_id,
            handicap: None,
            side,
            limit_order: Some(LimitOrder {
                size: stake,
                price,
                persistence_type,
                time_in_force: None,
                min_fill_size: None,
                bet_target_type: None,
                bet_target_size: None,
            }),
            limit_on_close_order: None,
            market_on_close_order: None,
        }],
        customer_ref: None,
        market_version: None,
        customer_strategy_ref: None,
        r#async: None,
    };

    let report = client.place_orders(&request).await?;

    println!("Result: {:?}", report.status);
    if let Some(error) = &report.error_code {
        println!("Error: {:?}", error);
    }
    if let Some(reports) = &report.instruction_reports {
        for ir in reports {
            println!("  Instruction: {:?}", ir.status);
            if let Some(bet_id) = &ir.bet_id {
                println!("  Bet ID: {}", bet_id);
            }
            if let Some(matched) = ir.size_matched {
                println!("  Size matched: £{:.2}", matched);
            }
            if let Some(avg) = ir.average_price_matched {
                println!("  Avg price matched: {:.2}", avg);
            }
            if let Some(err) = &ir.error_code {
                println!("  Error: {}", err);
            }
        }
    }

    Ok(())
}

async fn cmd_cancel(
    client: &BetfairClient,
    market_id: &str,
    bet_id: &str,
    amount: Option<f64>,
) -> Result<()> {
    println!("Cancelling bet {} on market {}", bet_id, market_id);
    if let Some(amt) = amount {
        println!("  Partial cancel: £{:.2}", amt);
    }

    let request = CancelOrdersRequest {
        market_id: Some(market_id.to_string()),
        instructions: Some(vec![CancelInstruction {
            bet_id: bet_id.to_string(),
            size_reduction: amount,
        }]),
        customer_ref: None,
    };

    let report = client.cancel_orders(&request).await?;
    println!("\nResult: {:?}", report.status);
    if let Some(error) = &report.error_code {
        println!("Error: {:?}", error);
    }
    if let Some(reports) = &report.instruction_reports {
        for ir in reports {
            println!("  Status: {:?}", ir.status);
            if let Some(cancelled) = ir.size_cancelled {
                println!("  Size cancelled: £{:.2}", cancelled);
            }
        }
    }

    Ok(())
}

async fn cmd_orders(client: &BetfairClient, market_id: Option<&str>) -> Result<()> {
    let request = ListCurrentOrdersRequest {
        bet_ids: None,
        market_ids: market_id.map(|id| vec![id.to_string()]),
        order_projection: Some(OrderProjection::All),
        date_range: None,
        order_by: None,
        from_record: None,
        record_count: None,
    };

    let report = client.list_current_orders(&request).await?;

    if report.current_orders.is_empty() {
        println!("No open orders.");
        return Ok(());
    }

    println!(
        "{:<16} {:<16} {:<12} {:>6} {:>8} {:>8} {:>8} {:>8} {:<8}",
        "Bet ID", "Market", "Selection", "Side", "Price", "Size", "Matched", "Remain", "Status"
    );
    println!("{}", "-".repeat(100));

    for order in &report.current_orders {
        println!(
            "{:<16} {:<16} {:<12} {:>6} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:<8?}",
            order.bet_id,
            order.market_id,
            order.selection_id,
            format!("{:?}", order.side),
            order.price_size.price,
            order.price_size.size,
            order.size_matched.unwrap_or(0.0),
            order.size_remaining.unwrap_or(0.0),
            order.status,
        );
    }

    println!(
        "\n{} order(s){}",
        report.current_orders.len(),
        if report.more_available {
            " (more available)"
        } else {
            ""
        }
    );

    Ok(())
}

async fn cmd_account(client: &BetfairClient) -> Result<()> {
    let (funds, details) = tokio::try_join!(
        client.get_account_funds(),
        client.get_account_details()
    )?;

    println!("Account Details:");
    if let (Some(first), Some(last)) = (&details.first_name, &details.last_name) {
        println!("  Name:     {} {}", first, last);
    }
    if let Some(cc) = &details.currency_code {
        println!("  Currency: {}", cc);
    }
    if let Some(country) = &details.country_code {
        println!("  Country:  {}", country);
    }
    if let Some(tz) = &details.timezone {
        println!("  Timezone: {}", tz);
    }

    println!("\nAccount Funds:");
    if let Some(balance) = funds.available_to_bet_balance {
        println!("  Available: £{:.2}", balance);
    }
    if let Some(exposure) = funds.exposure {
        println!("  Exposure:  £{:.2}", exposure);
    }
    if let Some(retained) = funds.retained_commission {
        println!("  Retained commission: £{:.2}", retained);
    }
    if let Some(discount) = funds.discount_rate {
        println!("  Discount rate: {:.1}%", discount);
    }

    Ok(())
}

async fn cmd_stream(
    session: &bpipe::auth::Session,
    config: &BetfairConfig,
    market_ids: Vec<String>,
    subscribe_orders: bool,
) -> Result<()> {
    let stream_config = StreamConfig {
        host: config.urls.stream_api.clone(),
        port: config.urls.stream_port,
        ..Default::default()
    };

    eprintln!("Connecting to streaming API...");
    let mut stream = StreamClient::connect(session, &stream_config)
        .await
        .context("Failed to connect to streaming API")?;

    if !market_ids.is_empty() {
        eprintln!("Subscribing to markets: {:?}", market_ids);
        stream
            .subscribe_markets(market_ids, None, Some(3))
            .await?;
    }

    if subscribe_orders {
        eprintln!("Subscribing to order changes...");
        stream.subscribe_orders().await?;
    }

    eprintln!("Streaming started. Press Ctrl+C to stop.\n");

    while let Some(msg) = stream.next_message().await {
        match msg {
            bpipe::types::streaming::StreamMessage::MarketChange(mcm) => {
                if let Some(changes) = &mcm.mc {
                    for mc in changes {
                        println!(
                            "[MCM] Market {} (vol: {:.0})",
                            mc.id,
                            mc.tv.unwrap_or(0.0)
                        );
                        if let Some(runners) = &mc.rc {
                            for rc in runners {
                                let mut parts = vec![format!("  Runner {}", rc.id)];
                                if let Some(ltp) = rc.ltp {
                                    parts.push(format!("LTP={:.2}", ltp));
                                }
                                if let Some(tv) = rc.tv {
                                    parts.push(format!("TV={:.0}", tv));
                                }
                                if let Some(atb) = &rc.atb {
                                    let backs: Vec<String> = atb
                                        .iter()
                                        .map(|v| {
                                            if v.len() >= 2 {
                                                format!("{:.2}×{:.2}", v[0], v[1])
                                            } else {
                                                format!("{:?}", v)
                                            }
                                        })
                                        .collect();
                                    parts.push(format!("BACK=[{}]", backs.join(", ")));
                                }
                                if let Some(atl) = &rc.atl {
                                    let lays: Vec<String> = atl
                                        .iter()
                                        .map(|v| {
                                            if v.len() >= 2 {
                                                format!("{:.2}×{:.2}", v[0], v[1])
                                            } else {
                                                format!("{:?}", v)
                                            }
                                        })
                                        .collect();
                                    parts.push(format!("LAY=[{}]", lays.join(", ")));
                                }
                                println!("{}", parts.join("  "));
                            }
                        }
                    }
                }
            }
            bpipe::types::streaming::StreamMessage::OrderChange(ocm) => {
                if let Some(changes) = &ocm.oc {
                    for oc in changes {
                        println!("[OCM] Market {}", oc.id);
                        if let Some(runners) = &oc.orc {
                            for orc in runners {
                                println!("  Runner {}", orc.id);
                                if let Some(orders) = &orc.uo {
                                    for o in orders {
                                        println!(
                                            "    Bet {} — side={:?} p={:?} s={:?} sm={:?} sr={:?} status={:?}",
                                            o.id,
                                            o.side,
                                            o.p,
                                            o.s,
                                            o.sm,
                                            o.sr,
                                            o.status
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            bpipe::types::streaming::StreamMessage::Status {
                status_code,
                error_code,
                error_message,
                ..
            } => {
                println!(
                    "[STATUS] code={:?} error={:?} msg={:?}",
                    status_code, error_code, error_message
                );
            }
            bpipe::types::streaming::StreamMessage::Connection { connection_id } => {
                println!("[CONNECTED] {}", connection_id);
            }
        }
    }

    eprintln!("Stream closed.");
    Ok(())
}

// ─── Helpers ────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

// ─── Scrape command ─────────────────────────────────────

async fn cmd_scrape(client: &BetfairClient, url: &str, upcoming_only: bool) -> Result<()> {

    eprintln!("Scraping predictions from {}...", url);
    let mut matches = scraper::scrape_predictions(url).await?;

    if upcoming_only {
        matches.retain(|m| !m.has_result);
    }

    eprintln!("Found {} match(es). Searching Betfair for events...\n", matches.len());

    // Get today's soccer events from Betfair to match against
    // Event type ID "1" = Soccer
    let today_filter = MarketFilter {
        event_type_ids: Some(vec!["1".to_string()]),
        ..Default::default()
    };
    let betfair_events = client.list_events(&today_filter).await?;
    eprintln!("Loaded {} Betfair soccer events to match against.\n", betfair_events.len());

    // Print header
    println!(
        "{:<45} {:<24} {:<6} {:>4} {:>4} {:>4}  {:<12} {:<30}",
        "MATCH", "DATE", "TIP", "1%", "X%", "2%", "BF EVENT ID", "BF EVENT NAME"
    );
    println!("{}", "─".repeat(140));

    let mut matched = 0;
    let mut unmatched = 0;

    for m in &matches {
        let display_match = format!("{} v {}", m.home_team, m.away_team);
        let display_match = truncate(&display_match, 44);

        // Try to find a matching Betfair event
        // Strategy: fuzzy match by checking if both team names appear in the Betfair event name
        let bf_match = find_betfair_event(&betfair_events, &m.home_team, &m.away_team);

        let (bf_id, bf_name) = match &bf_match {
            Some(ev) => {
                matched += 1;
                (ev.event.id.clone(), truncate(&ev.event.name, 29).to_string())
            }
            None => {
                unmatched += 1;
                ("—".to_string(), "—".to_string())
            }
        };

        let status_marker = if m.has_result { "✓" } else { " " };

        println!(
            "{}{:<44} {:<24} {:<6} {:>4} {:>4} {:>4}  {:<12} {:<30}",
            status_marker,
            display_match,
            m.datetime,
            m.tip,
            m.prob_home.map_or("—".to_string(), |v| v.to_string()),
            m.prob_draw.map_or("—".to_string(), |v| v.to_string()),
            m.prob_away.map_or("—".to_string(), |v| v.to_string()),
            bf_id,
            bf_name,
        );
    }

    println!("\n{} matches total, {} matched to Betfair, {} unmatched",
        matches.len(), matched, unmatched);

    Ok(())
}

/// Find a Betfair event matching the given home and away team names.
///
/// Uses fuzzy matching: normalizes both statarea team names and Betfair event names
/// to lowercase, strips common suffixes like "(w)", "FC", "CF", etc.,
/// then checks if both teams appear in the Betfair event name.
fn find_betfair_event<'a>(
    events: &'a [EventResult],
    home: &str,
    away: &str,
) -> Option<&'a EventResult> {
    let home_norm = normalize_team(home);
    let away_norm = normalize_team(away);

    // First: exact substring match (both teams in event name)
    for ev in events {
        let ev_name = ev.event.name.to_lowercase();
        if ev_name.contains(&home_norm) && ev_name.contains(&away_norm) {
            return Some(ev);
        }
    }

    // Second pass: try matching with just the first word of each team name
    // (e.g., "Vikingur Reykjavik" might be "Vikingur" on Betfair)
    let home_short = home_norm.split_whitespace().next().unwrap_or(&home_norm);
    let away_short = away_norm.split_whitespace().next().unwrap_or(&away_norm);

    // Only try short match if the short names are ≥3 chars to avoid false positives
    if home_short.len() >= 3 && away_short.len() >= 3 {
        for ev in events {
            let ev_name = ev.event.name.to_lowercase();
            if ev_name.contains(home_short) && ev_name.contains(away_short) {
                return Some(ev);
            }
        }
    }

    None
}

/// Normalize a team name for fuzzy matching.
fn normalize_team(name: &str) -> String {
    let s = name.to_lowercase();
    // Remove common suffixes/markers
    let s = s
        .replace("(w)", "")
        .replace(" fc", "")
        .replace(" cf", "")
        .replace(" sc", "")
        .replace(" ec", "")
        .replace(" ac", "")
        .replace(" afc", "");
    s.trim().to_string()
}

// ─── Match Events command ───────────────────────────────

async fn cmd_match_events(client: &BetfairClient, input: &PathBuf, upcoming_only: bool, output: &PathBuf) -> Result<()> {
    use std::fmt::Write as _;

    let json = std::fs::read_to_string(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;

    let mut matches: Vec<ScrapedMatch> = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse {}", input.display()))?;

    if upcoming_only {
        matches.retain(|m| !m.has_result);
    }

    matches.sort_by(|a, b| a.datetime.cmp(&b.datetime));

    eprintln!("Loaded {} match(es) from {}. Searching Betfair for events...\n",
        matches.len(), input.display());

    // Get today's soccer events from Betfair to match against
    let today_filter = MarketFilter {
        event_type_ids: Some(vec!["1".to_string()]),
        ..Default::default()
    };
    let betfair_events = client.list_events(&today_filter).await?;
    eprintln!("Loaded {} Betfair soccer events to match against.\n", betfair_events.len());

    let header = format!(
        "{:<45} {:<24} {:<6} {:>4} {:>4} {:>4}  {:<12} {:<30}",
        "MATCH", "DATE", "TIP", "1%", "X%", "2%", "BF EVENT ID", "BF EVENT NAME"
    );
    let separator = "─".repeat(140);
    println!("{}", header);
    println!("{}", separator);

    let mut buf = String::new();
    writeln!(buf, "{}", header).unwrap();
    writeln!(buf, "{}", separator).unwrap();

    let mut matched = 0;
    let mut unmatched = 0;

    for m in &matches {
        let bf_match = find_betfair_event(&betfair_events, &m.home_team, &m.away_team);

        match bf_match {
            Some(ev) => {
                matched += 1;
                let display_match = format!("{} v {}", m.home_team, m.away_team);
                let display_match = truncate(&display_match, 44);
                let status_marker = if m.has_result { "✓" } else { " " };
                let line = format!(
                    "{}{:<44} {:<24} {:<6} {:>4} {:>4} {:>4}  {:<12} {:<30}",
                    status_marker,
                    display_match,
                    m.datetime,
                    m.tip,
                    m.prob_home.map_or("—".to_string(), |v| v.to_string()),
                    m.prob_draw.map_or("—".to_string(), |v| v.to_string()),
                    m.prob_away.map_or("—".to_string(), |v| v.to_string()),
                    ev.event.id,
                    truncate(&ev.event.name, 29),
                );
                println!("{}", line);
                writeln!(buf, "{}", line).unwrap();
            }
            None => {
                unmatched += 1;
            }
        }
    }

    let summary = format!("\n{} matched to Betfair, {} unmatched (out of {} scraped)",
        matched, unmatched, matches.len());
    println!("{}", summary);
    writeln!(buf, "{}", summary).unwrap();

    std::fs::write(output, &buf)
        .with_context(|| format!("Failed to write {}", output.display()))?;
    eprintln!("Saved to {}", output.display());

    Ok(())
}
