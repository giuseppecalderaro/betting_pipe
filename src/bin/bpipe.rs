use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
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

        /// Output file for the matched table (default: matched_YYYYMMDD_HHMMSS.txt)
        #[arg(long, short)]
        output: Option<PathBuf>,
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
        Command::MatchEvents { input, upcoming, output } => {
            let output = output.unwrap_or_else(|| {
                let ts = Local::now().format("%Y%m%d_%H%M%S");
                PathBuf::from(format!("matched_{ts}.txt"))
            });
            cmd_match_events(&client, &input, upcoming, &output).await?
        }
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
/// Uses multi-pass fuzzy matching with token-based scoring:
/// 1. Exact substring match after normalization
/// 2. Token overlap scoring (best match above threshold wins)
/// 3. Alias expansion for known team name variants
fn find_betfair_event<'a>(
    events: &'a [EventResult],
    home: &str,
    away: &str,
) -> Option<&'a EventResult> {
    let home_norm = normalize_team(home);
    let away_norm = normalize_team(away);

    // Pass 1: exact substring match (both normalized teams in event name)
    for ev in events {
        let ev_name = normalize_team(&ev.event.name);
        if ev_name.contains(&home_norm) && ev_name.contains(&away_norm) {
            return Some(ev);
        }
    }

    // Pass 2: alias-expanded substring match
    let home_aliases = team_aliases(&home_norm);
    let away_aliases = team_aliases(&away_norm);
    for ev in events {
        let ev_name = normalize_team(&ev.event.name);
        let home_ok = home_aliases.iter().any(|a| ev_name.contains(a.as_str()));
        let away_ok = away_aliases.iter().any(|a| ev_name.contains(a.as_str()));
        if home_ok && away_ok {
            return Some(ev);
        }
    }

    // Pass 3: token-based scoring — find best match above threshold
    let home_tokens = tokenize(&home_norm);
    let away_tokens = tokenize(&away_norm);
    let home_alias_tokens: Vec<Vec<String>> = home_aliases.iter().map(|a| tokenize(a)).collect();
    let away_alias_tokens: Vec<Vec<String>> = away_aliases.iter().map(|a| tokenize(a)).collect();

    let mut best: Option<(f64, &'a EventResult)> = None;

    for ev in events {
        let ev_name = normalize_team(&ev.event.name);
        // Betfair event names are typically "Team A v Team B"
        let ev_parts: Vec<&str> = ev_name.splitn(2, " v ").collect();
        if ev_parts.len() != 2 {
            continue;
        }
        let ev_home_tokens = tokenize(ev_parts[0]);
        let ev_away_tokens = tokenize(ev_parts[1]);

        // Score: best token overlap for home vs ev_home + away vs ev_away
        let home_score = best_token_score(&home_tokens, &home_alias_tokens, &ev_home_tokens);
        let away_score = best_token_score(&away_tokens, &away_alias_tokens, &ev_away_tokens);

        // Also try swapped order (statarea order might differ from Betfair)
        let home_score_swap = best_token_score(&home_tokens, &home_alias_tokens, &ev_away_tokens);
        let away_score_swap = best_token_score(&away_tokens, &away_alias_tokens, &ev_home_tokens);

        let score = (home_score * away_score).max(home_score_swap * away_score_swap);

        // Threshold: both teams must have ≥0.5 individual and combined ≥0.36
        let (ind_home, ind_away) = if home_score * away_score >= home_score_swap * away_score_swap {
            (home_score, away_score)
        } else {
            (home_score_swap, away_score_swap)
        };

        if ind_home >= 0.5 && ind_away >= 0.5 && score > best.map_or(0.0, |(s, _)| s) {
            best = Some((score, ev));
        }
    }

    best.map(|(_, ev)| ev)
}

/// Compute the best token overlap score between a team name (+ aliases) and an event side.
fn best_token_score(tokens: &[String], alias_token_sets: &[Vec<String>], ev_tokens: &[String]) -> f64 {
    let mut best = token_overlap(tokens, ev_tokens);
    for alias_tokens in alias_token_sets {
        best = best.max(token_overlap(alias_tokens, ev_tokens));
    }
    best
}

/// Token overlap score: |intersection| / max(|a|, |b|).
/// For single-token teams, also consider substring matching.
fn token_overlap(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    // Exact token intersection
    let intersection = a.iter().filter(|t| b.contains(t)).count();

    // For remaining unmatched tokens, check substring containment (≥4 chars)
    let mut substr_matches = 0.0;
    for ta in a.iter() {
        if b.contains(ta) {
            continue; // Already counted
        }
        if ta.len() >= 4 {
            for tb in b.iter() {
                if tb.len() >= 4 && (tb.contains(ta.as_str()) || ta.contains(tb.as_str())) {
                    substr_matches += 0.8; // Partial credit for substring match
                    break;
                }
            }
        }
    }

    let score = (intersection as f64 + substr_matches) / a.len().max(b.len()) as f64;
    score.min(1.0)
}

/// Tokenize a normalized name into meaningful words, filtering noise words.
fn tokenize(name: &str) -> Vec<String> {
    name.split_whitespace()
        .filter(|w| w.len() >= 2) // Drop single-char tokens
        .filter(|w| !NOISE_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

const NOISE_WORDS: &[&str] = &[
    "de", "da", "do", "dos", "du", "la", "le", "les", "el", "al", "di", "del",
    "van", "von", "st", "saint", "san", "the",
];

/// Normalize a team name for fuzzy matching.
fn normalize_team(name: &str) -> String {
    let s = name.to_lowercase();

    // Remove content in parentheses: (W), (R), etc.
    let s = s
        .replace("(w)", "")
        .replace("(r)", "")
        .replace("(n)", "");

    // Remove common club type suffixes and prefixes
    let stripped: String = s
        // Suffixes
        .replace(" fc", "")
        .replace(" cf", "")
        .replace(" sc", "")
        .replace(" ec", "")
        .replace(" ac", "")
        .replace(" afc", "")
        .replace(" fk", "")
        .replace(" bk", "")
        .replace(" sk", "")
        .replace(" if", "")
        .replace(" ff", "")
        .replace(" ik", "")
        .replace(" pk", "")
        .replace(" cd", "")
        .replace(" ca", "")
        .replace(" cs", "")
        .replace(" as", "")
        .replace(" us", "")
        .replace(" ss", "")
        .replace(" ssd", "")
        .replace(" gf", "")
        .replace(" rj", "")
        .replace(" mg", "")
        .replace(" sp", "")
        .replace(" se", "")
        .replace(" rs", "");

    // Remove leading prefixes (e.g., "FC Basel", "NK Dinamo Zagreb", "SC Braga")
    let stripped = strip_leading_prefix(&stripped);

    // Remove dots and extra punctuation
    let stripped = stripped
        .replace('.', " ")
        .replace('-', " ")
        .replace('\'', "")
        .replace('/', " ");

    // Collapse multiple spaces
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip leading club-type prefixes like "FC", "NK", "SC", "KV", etc.
fn strip_leading_prefix(name: &str) -> String {
    let prefixes = [
        "fc ", "fk ", "nk ", "sc ", "sk ", "kv ", "sv ", "bv ", "vfb ", "vfl ",
        "tsv ", "bsc ", "fsv ", "ssv ", "msv ", "tsg ", "rb ", "rcd ",
        "cd ", "ca ", "cs ", "cf ", "as ", "us ", "ss ", "aek ", "rc ", "rsc ",
        "gd ", "gf ", "ud ", "uc ",
    ];
    let trimmed = name.trim();
    for pfx in &prefixes {
        if trimmed.starts_with(pfx) {
            let rest = trimmed[pfx.len()..].trim();
            if rest.len() >= 3 {
                return rest.to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Return a list of known aliases for a (normalized) team name.
/// Includes the original name plus any known variants.
fn team_aliases(normalized: &str) -> Vec<String> {
    let mut aliases = vec![normalized.to_string()];

    // Well-known name aliases (statarea name → betfair variants and vice versa)
    let alias_map: &[&[&str]] = &[
        &["inter", "inter milan", "internazionale"],
        &["ac milan", "milan"],
        &["betis", "real betis"],
        &["atletico madrid", "atl madrid", "atletico"],
        &["atletico mineiro", "atl mineiro"],
        &["atletico goianiense", "atl goianiense"],
        &["atletico tucuman", "atl tucuman"],
        &["atletico nacional", "atl nacional"],
        &["paris saint germain", "psg", "paris sg", "paris st germain"],
        &["bayern munich", "bayern munchen", "bayern"],
        &["1899 hoffenheim", "hoffenheim", "tsg hoffenheim"],
        &["bayer leverkusen", "leverkusen"],
        &["borussia dortmund", "dortmund", "bvb"],
        &["borussia monchengladbach", "monchengladbach", "gladbach", "bmg"],
        &["hertha berlin", "hertha"],
        &["eintracht frankfurt", "frankfurt", "ein frankfurt"],
        &["schalke 04", "schalke"],
        &["werder bremen", "bremen"],
        &["vfb stuttgart", "stuttgart"],
        &["1 fsv mainz 05", "mainz", "mainz 05"],
        &["union berlin", "union"],
        &["arminia bielefeld", "bielefeld"],
        &["fortuna dusseldorf", "dusseldorf"],
        &["eintracht braunschweig", "braunschweig"],
        &["karlsruher", "karlsruhe"],
        &["koln", "cologne", "fc koln"],
        &["rb leipzig", "leipzig", "rasenballsport leipzig"],
        &["sporting lisbon", "sporting", "sporting cp", "sporting clube de portugal"],
        &["benfica", "sl benfica"],
        &["porto", "fc porto"],
        &["braga", "sc braga", "sporting braga"],
        &["deportivo alaves", "alaves"],
        &["deportivo la coruna", "deportivo"],
        &["celta vigo", "celta"],
        &["real sociedad", "sociedad"],
        &["athletic club", "athletic bilbao", "ath bilbao"],
        &["wolverhampton", "wolves", "wolverhampton wanderers"],
        &["tottenham", "tottenham hotspur", "spurs"],
        &["newcastle", "newcastle united", "newcastle utd"],
        &["manchester united", "man united", "man utd"],
        &["manchester city", "man city"],
        &["west ham", "west ham united", "west ham utd"],
        &["sheffield united", "sheffield utd", "sheff utd"],
        &["sheffield wednesday", "sheffield wed", "sheff wed"],
        &["nottingham forest", "nottm forest", "nott forest"],
        &["leicester", "leicester city"],
        &["aston villa", "villa"],
        &["crystal palace", "c palace"],
        &["queens park rangers", "qpr"],
        &["olympiakos piraeus", "olympiakos", "olympiacos"],
        &["aris thessalonikis", "aris thessaloniki", "aris"],
        &["panathinaikos", "panathinaikos athens"],
        &["paok", "paok thessaloniki", "paok salonika"],
        &["aek athens", "aek"],
        &["dinamo zagreb", "nk dinamo zagreb"],
        &["hajduk split", "hajduk"],
        &["rijeka", "hnk rijeka"],
        &["osijek", "nk osijek"],
        &["varazdin", "nk varazdin"],
        &["red star belgrade", "crvena zvezda", "red star"],
        &["partizan belgrade", "partizan"],
        &["galatasaray", "galatasaray sk"],
        &["fenerbahce", "fenerbahce sk"],
        &["besiktas", "besiktas jk"],
        &["basaksehir", "istanbul basaksehir"],
        &["eyupspor", "eyup"],
        &["midtjylland", "fc midtjylland"],
        &["copenhagen", "fc copenhagen", "kobenhavn"],
        &["odense", "odense bk", "ob"],
        &["agf", "agf aarhus", "aarhus"],
        &["basel", "fc basel", "fc basel 1893"],
        &["zurich", "fc zurich"],
        &["servette", "servette fc"],
        &["lausanne", "fc lausanne sport", "fc lausanne"],
        &["st gallen", "fc st gallen"],
        &["thun", "fc thun"],
        &["viktoria plzen", "plzen", "viktoria"],
        &["slovan liberec", "liberec"],
        &["sparta praha", "sparta prague"],
        &["slavia praha", "slavia prague"],
        &["olympique lyonnais", "lyon", "ol"],
        &["olympique marseille", "marseille", "om"],
        &["as monaco", "monaco"],
        &["as saint etienne", "saint etienne", "st etienne"],
        &["losc lille", "lille"],
        &["stade rennais", "rennes"],
        &["montpellier hsc", "montpellier"],
        &["rc strasbourg", "strasbourg"],
        &["stade brestois", "brest"],
        &["girondins bordeaux", "bordeaux"],
        &["ogc nice", "nice"],
        &["angers sco", "angers"],
        &["fc nantes", "nantes"],
        &["fc toulouse", "toulouse"],
        &["ajaccio", "ac ajaccio", "gazelec ajaccio"],
        &["le havre", "le havre ac"],
        &["paris fc", "paris"],
        &["auxerre", "aj auxerre"],
        &["club america", "america"],
        &["u n a m pumas", "pumas unam", "pumas"],
        &["queretaro", "queretaro fc"],
        &["fc juarez", "juarez"],
        &["union st gilloise", "union sg", "royale union saint gilloise"],
        &["kv mechelen", "mechelen"],
        &["antwerp", "royal antwerp"],
        &["la louviere", "louviere"],
        &["club africain", "club africain tunis"],
        &["sfaxien", "cs sfaxien"],
        &["bizertin", "ca bizertin"],
        &["vasco da gama", "vasco"],
        &["botafogo", "botafogo rj"],
        &["sampaio correa", "sampaio correa rj"],
        &["america mineiro", "america mg"],
        &["bohemians", "bohemian fc", "bohemians dublin"],
        &["st patricks athletic", "st patricks", "st pats"],
        &["olimpija ljubljana", "olimpija"],
        &["celje", "nk celje"],
        &["mura", "ns mura"],
        &["maribor", "nk maribor"],
        &["koper", "fc koper"],
        &["river plate", "river"],
        &["newells old boys", "newells"],
        &["defensa y justicia", "defensa"],
        &["belgrano cordoba", "belgrano"],
        &["sarmiento junin", "sarmiento"],
        &["olimpia", "cd olimpia"],
        &["lobos upnfm", "lobos"],
        &["apoel", "apoel nicosia"],
        &["fortaleza", "fortaleza fc", "fortaleza ce"],
        &["america de cali", "america cali"],
        &["santa fe", "ind santa fe", "independiente santa fe"],
        &["junior", "junior barranquilla", "atletico junior"],
        &["chico", "boyaca chico"],
        &["altach", "scr altach", "rheindorf altach"],
        &["bw linz", "fc bw linz", "blau weiss linz"],
        &["botosani", "fc botosani"],
        &["metaloglobus", "metaloglobus bucharest"],
        &["malaga", "malaga cf"],
        &["cultural leonesa", "cultural"],
        &["esperance", "esperance st", "esperance tunis", "esperance sportive de tunis"],
        &["stade malien", "stade malien bamako"],
        &["pachuca", "cf pachuca"],
        &["toluca", "deportivo toluca"],
        &["club guarani", "guarani"],
        &["libertad", "club libertad"],
        &["monterrey", "cf monterrey"],
        &["huachipato", "cd huachipato"],
        &["u de chile", "universidad de chile"],
        &["cruz azul", "cd cruz azul"],
        &["panserraikos", "panserraikos fc"],
        &["ofi", "ofi crete"],
        &["levadiakos", "levadiakos fc"],
        &["como", "como 1907"],
    ];

    for group in alias_map {
        let group_lower: Vec<String> = group.iter().map(|s| s.to_string()).collect();
        if group_lower.iter().any(|a| a == normalized) {
            for a in &group_lower {
                if a != normalized {
                    aliases.push(a.clone());
                }
            }
            break;
        }
        // Also check if the normalized name *contains* any alias or vice versa
        for a in &group_lower {
            if normalized.contains(a.as_str()) || a.contains(normalized) {
                for other in &group_lower {
                    if !aliases.contains(other) {
                        aliases.push(other.clone());
                    }
                }
                break;
            }
        }
    }

    // Generate short-name variant: first meaningful word if ≥4 chars
    let words: Vec<&str> = normalized.split_whitespace().collect();
    if words.len() > 1 {
        // Significant first word (skip noise/generic prefixes)
        for w in &words {
            if w.len() >= 4 && !NOISE_WORDS.contains(w) {
                let short = w.to_string();
                if !aliases.contains(&short) {
                    aliases.push(short);
                }
                break;
            }
        }
        // Also try last significant word
        for w in words.iter().rev() {
            if w.len() >= 4 && !NOISE_WORDS.contains(w) {
                let short = w.to_string();
                if !aliases.contains(&short) {
                    aliases.push(short);
                }
                break;
            }
        }
    }

    aliases
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
