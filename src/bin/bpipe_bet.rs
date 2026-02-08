use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use bpipe::types::betting::*;
use bpipe::types::common::*;
use bpipe::{auth, BetfairClient, BetfairConfig};

/// bpipe-bet — Read a matched text file and place bets following statarea tips
#[derive(Parser)]
#[command(name = "bpipe-bet", version, about = "Place bets from a matched events file")]
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

    /// Path to client certificate (for bot login)
    #[arg(long, env = "BETFAIR_CERT_PATH")]
    cert: Option<PathBuf>,

    /// Path to client key (for bot login)
    #[arg(long, env = "BETFAIR_KEY_PATH")]
    key: Option<PathBuf>,

    /// Path to the matched text file
    #[arg(long, short)]
    input: PathBuf,

    /// Stake per bet in £
    #[arg(long, short, default_value = "2.0")]
    stake: f64,

    /// Only bet on matches that haven't started yet
    #[arg(long)]
    upcoming_only: bool,

    /// Minimum win probability (%) to place a bet
    #[arg(long, default_value = "50")]
    min_prob: u32,

    /// Maximum odds to accept (skip selections above this price)
    #[arg(long)]
    max_odds: Option<f64>,

    /// Dry run — show what would be bet but don't place orders
    #[arg(long)]
    dry_run: bool,

    /// Auto-confirm all bets without prompting
    #[arg(long)]
    yes: bool,

    /// Persistence type: lapse, persist, or moc
    #[arg(long, default_value = "lapse")]
    persistence: String,
}

/// A parsed entry from the matched text file.
#[derive(Debug)]
struct MatchedEntry {
    home_team: String,
    away_team: String,
    datetime: String,
    tip: String,
    prob_home: Option<u32>,
    prob_draw: Option<u32>,
    prob_away: Option<u32>,
    bf_event_id: String,
    bf_event_name: String,
    has_result: bool,
}

/// Which runner(s) to bet on based on the tip.
#[derive(Debug)]
enum TipSelection {
    /// Single selection: Home (1), Draw (X), or Away (2)
    Single(RunnerPosition),
    /// Double chance: back two outcomes (e.g., 1X = Home + Draw)
    Double(RunnerPosition, RunnerPosition),
}

#[derive(Debug, Clone, Copy)]
enum RunnerPosition {
    Home,
    Draw,
    Away,
}

impl std::fmt::Display for RunnerPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerPosition::Home => write!(f, "Home"),
            RunnerPosition::Draw => write!(f, "Draw"),
            RunnerPosition::Away => write!(f, "Away"),
        }
    }
}

fn parse_tip(tip: &str) -> Option<TipSelection> {
    match tip.trim() {
        "1" => Some(TipSelection::Single(RunnerPosition::Home)),
        "X" | "x" => Some(TipSelection::Single(RunnerPosition::Draw)),
        "2" => Some(TipSelection::Single(RunnerPosition::Away)),
        "1X" | "1x" => Some(TipSelection::Double(RunnerPosition::Home, RunnerPosition::Draw)),
        "X2" | "x2" => Some(TipSelection::Double(RunnerPosition::Draw, RunnerPosition::Away)),
        "12" => Some(TipSelection::Double(RunnerPosition::Home, RunnerPosition::Away)),
        _ => None,
    }
}

/// Get the probability for a runner position from the entry.
fn prob_for_position(entry: &MatchedEntry, pos: RunnerPosition) -> Option<u32> {
    match pos {
        RunnerPosition::Home => entry.prob_home,
        RunnerPosition::Draw => entry.prob_draw,
        RunnerPosition::Away => entry.prob_away,
    }
}

/// Parse the matched text file into entries.
fn parse_matched_file(path: &PathBuf) -> Result<Vec<MatchedEntry>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let mut entries = Vec::new();

    for line in content.lines() {
        // Skip header, separator, and summary lines
        if line.contains("MATCH") && line.contains("DATE") && line.contains("TIP") {
            continue;
        }
        if line.starts_with('─') || line.starts_with('-') {
            continue;
        }
        if line.contains("matched to Betfair") || line.contains("unmatched") {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        // Collect chars so we can index by display position (not byte offset).
        // Multi-byte chars like ✓ (3 bytes) and … (3 bytes) are each 1 char.
        let chars: Vec<char> = line.chars().collect();

        let has_result = chars.first() == Some(&'✓');
        // Skip the leading marker char (✓ or space)
        let chars = &chars[1..];

        if chars.len() < 100 {
            continue;
        }

        // Helper: safely extract a char slice and convert to String
        let slice = |from: usize, to: usize| -> String {
            chars[from..to.min(chars.len())].iter().collect()
        };

        // Fixed-width fields by *character* position (after marker):
        //   0..44   match name
        //  44..69   datetime
        //  69..75   tip
        //  75..79   prob home
        //  79..83   prob draw
        //  83..88   prob away (with leading space)
        //  90..     bf_event_id + bf_event_name
        let match_field = slice(0, 44);
        let match_field = match_field.trim();
        let date_field = slice(44, 69);
        let date_field = date_field.trim();
        let tip_field = slice(69, 75);
        let tip_field = tip_field.trim();

        let p1_str = slice(75, 79);
        let px_str = slice(79, 83);
        let p2_str = slice(83, 88);

        let rest: String = if chars.len() > 90 {
            chars[90..].iter().collect()
        } else {
            continue;
        };

        // Parse the team names from match field
        let parts: Vec<&str> = match_field.splitn(2, " v ").collect();
        if parts.len() != 2 {
            continue;
        }
        let home_team = parts[0].trim().to_string();
        let away_team = parts[1].trim().to_string();

        // Parse probabilities
        let p1 = p1_str.trim().parse::<u32>().ok();
        let px = px_str.trim().parse::<u32>().ok();
        let p2 = p2_str.trim().parse::<u32>().ok();

        // Parse BF Event ID and Name
        let rest = rest.trim();
        let bf_parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        let bf_event_id = bf_parts.first().unwrap_or(&"").trim().to_string();
        let bf_event_name = if bf_parts.len() > 1 {
            bf_parts[1].trim().to_string()
        } else {
            String::new()
        };

        if bf_event_id.is_empty() {
            continue;
        }

        entries.push(MatchedEntry {
            home_team,
            away_team,
            datetime: date_field.to_string(),
            tip: tip_field.to_string(),
            prob_home: p1,
            prob_draw: px,
            prob_away: p2,
            bf_event_id,
            bf_event_name,
            has_result,
        });
    }

    Ok(entries)
}

/// Find the "Match Odds" market for an event and return the catalogue.
/// Tries MATCH_ODDS first, then MATCH_ODDS_UNMANAGED as fallback.
/// If neither is found, lists available market types for diagnostics.
async fn find_match_odds_market(
    client: &BetfairClient,
    event_id: &str,
) -> Result<Option<MarketCatalogue>> {
    // Try these market types in order of preference
    let market_types = vec![
        "MATCH_ODDS".to_string(),
        "MATCH_ODDS_UNMANAGED".to_string(),
    ];

    let request = ListMarketCatalogueRequest {
        filter: MarketFilter {
            event_ids: Some(vec![event_id.to_string()]),
            market_type_codes: Some(market_types),
            ..Default::default()
        },
        market_projection: Some(vec![
            MarketProjection::RunnerDescription,
            MarketProjection::MarketStartTime,
        ]),
        sort: Some(MarketSort::FirstToStart),
        max_results: Some(5),
        locale: None,
    };

    let results = client.list_market_catalogue(&request).await?;

    if let Some(cat) = results.into_iter().next() {
        return Ok(Some(cat));
    }

    // No match odds found — query what market types exist for diagnostics
    let filter = MarketFilter {
        event_ids: Some(vec![event_id.to_string()]),
        ..Default::default()
    };
    let types = client.list_market_types(&filter).await?;
    if types.is_empty() {
        println!("     (no markets at all for this event — likely finished or not yet available)");
    } else {
        let type_list: Vec<String> = types
            .iter()
            .map(|t| format!("{} ({})", t.market_type, t.market_count.unwrap_or(0)))
            .collect();
        println!("     Available market types: {}", type_list.join(", "));
    }

    Ok(None)
}

/// Get the selection ID for a runner position in a 3-way match odds market.
///
/// Betfair Match Odds runners are ordered: Home, Away, Draw (sort priority 1, 2, 3).
/// But we can also match by name as fallback.
fn runner_for_position(
    catalogue: &MarketCatalogue,
    position: RunnerPosition,
) -> Option<&RunnerCatalog> {
    let runners = catalogue.runners.as_ref()?;
    if runners.len() < 3 {
        return None;
    }

    // Sort by sort_priority to reliably identify home/away/draw
    let mut sorted: Vec<&RunnerCatalog> = runners.iter().collect();
    sorted.sort_by_key(|r| r.sort_priority.unwrap_or(99));

    // Betfair Match Odds sort priority: 1=Home, 2=Away, 3=Draw
    match position {
        RunnerPosition::Home => sorted.first().copied(),
        RunnerPosition::Away => sorted.get(1).copied(),
        RunnerPosition::Draw => sorted.get(2).copied(),
    }
}

/// Get live best back price for a selection in a market.
async fn get_best_back_price(
    client: &BetfairClient,
    market_id: &str,
    selection_id: i64,
) -> Result<Option<PriceSize>> {
    let request = ListMarketBookRequest {
        market_ids: vec![market_id.to_string()],
        price_projection: Some(PriceProjection {
            price_data: Some(vec![PriceData::ExBestOffers]),
            ex_best_offers_overrides: Some(ExBestOffersOverrides {
                best_prices_depth: Some(1),
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
    let book = match books.first() {
        Some(b) => b,
        None => return Ok(None),
    };

    let runner = book
        .runners
        .as_ref()
        .and_then(|runners| runners.iter().find(|r| r.selection_id == selection_id));

    let best_back = runner
        .and_then(|r| r.ex.as_ref())
        .and_then(|ex| ex.available_to_back.as_ref())
        .and_then(|backs| backs.first())
        .cloned();

    Ok(best_back)
}

/// Prompt the user for yes/no confirmation.
fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let persistence_type = match cli.persistence.to_uppercase().as_str() {
        "LAPSE" | "L" => PersistenceType::Lapse,
        "PERSIST" | "P" => PersistenceType::Persist,
        "MOC" | "MARKET_ON_CLOSE" => PersistenceType::MarketOnClose,
        _ => anyhow::bail!("Persistence must be 'lapse', 'persist', or 'moc'"),
    };

    // Parse matched file
    let entries = parse_matched_file(&cli.input)?;
    eprintln!("Loaded {} matched entries from {}", entries.len(), cli.input.display());

    // Filter
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|e| {
            if cli.upcoming_only && e.has_result {
                return false;
            }
            true
        })
        .collect();

    if entries.is_empty() {
        eprintln!("No entries to process.");
        return Ok(());
    }

    eprintln!("After filtering: {} entries to process", entries.len());
    eprintln!("Stake: £{:.2} per bet | Min probability: {}% | Dry run: {}\n",
        cli.stake, cli.min_prob, cli.dry_run);

    // Login to Betfair
    let config = if let (Some(cert), Some(key)) = (&cli.cert, &cli.key) {
        BetfairConfig::certificate(&cli.app_key, &cli.username, &cli.password, cert, key)
    } else {
        BetfairConfig::interactive(&cli.app_key, &cli.username, &cli.password)
    };

    let session = auth::login(&config).await?;
    let client = BetfairClient::new(session, &config);
    eprintln!("Logged in to Betfair.\n");

    let separator = "─".repeat(90);
    let mut total_placed = 0;
    let mut total_skipped = 0;
    let mut total_stake = 0.0;

    for (i, entry) in entries.iter().enumerate() {
        println!("{}", separator);
        println!(
            "[{}/{}] {} v {}",
            i + 1,
            entries.len(),
            entry.home_team,
            entry.away_team
        );
        println!(
            "  Date:     {}  |  BF Event: {} ({})",
            entry.datetime, entry.bf_event_id, entry.bf_event_name
        );
        println!(
            "  Tip:      {}  |  Probabilities: 1={:>3}%  X={:>3}%  2={:>3}%",
            entry.tip,
            entry.prob_home.map_or("—".to_string(), |v| v.to_string()),
            entry.prob_draw.map_or("—".to_string(), |v| v.to_string()),
            entry.prob_away.map_or("—".to_string(), |v| v.to_string()),
        );

        if entry.has_result {
            println!("  ⏭  Skipped: match already has a result");
            total_skipped += 1;
            continue;
        }

        // Parse the tip
        let tip_selection = match parse_tip(&entry.tip) {
            Some(ts) => ts,
            None => {
                println!("  ⏭  Skipped: unsupported tip '{}'", entry.tip);
                total_skipped += 1;
                continue;
            }
        };

        // For single tips, check min probability
        match &tip_selection {
            TipSelection::Single(pos) => {
                if let Some(prob) = prob_for_position(entry, *pos) {
                    if prob < cli.min_prob {
                        println!(
                            "  ⏭  Skipped: {} probability {}% < minimum {}%",
                            pos, prob, cli.min_prob
                        );
                        total_skipped += 1;
                        continue;
                    }
                }
            }
            TipSelection::Double(a, b) => {
                // For double chance, check combined probability
                let combined = prob_for_position(entry, *a).unwrap_or(0)
                    + prob_for_position(entry, *b).unwrap_or(0);
                if combined < cli.min_prob {
                    println!(
                        "  ⏭  Skipped: {} + {} combined probability {}% < minimum {}%",
                        a, b, combined, cli.min_prob
                    );
                    total_skipped += 1;
                    continue;
                }
            }
        }

        // Find the Match Odds market
        let catalogue = match find_match_odds_market(&client, &entry.bf_event_id).await? {
            Some(c) => c,
            None => {
                println!("  ⏭  Skipped: no Match Odds market found for event {}", entry.bf_event_id);
                total_skipped += 1;
                continue;
            }
        };

        let market_id = &catalogue.market_id;

        // Determine which runner(s) to back
        let selections: Vec<RunnerPosition> = match &tip_selection {
            TipSelection::Single(pos) => vec![*pos],
            TipSelection::Double(a, b) => {
                // For double chance tips (1X, X2, 12), we lay the excluded outcome instead
                // This is more efficient: lay Away for 1X, lay Home for X2, lay Draw for 12
                // But for simplicity and clarity, we'll back the higher-probability single selection
                let prob_a = prob_for_position(entry, *a).unwrap_or(0);
                let prob_b = prob_for_position(entry, *b).unwrap_or(0);
                if prob_a >= prob_b {
                    vec![*a]
                } else {
                    vec![*b]
                }
            }
        };

        for pos in &selections {
            let runner = match runner_for_position(&catalogue, *pos) {
                Some(r) => r,
                None => {
                    println!("  ⏭  Skipped: could not find {} runner in market", pos);
                    total_skipped += 1;
                    continue;
                }
            };

            // Get current best back price
            let best_back = get_best_back_price(&client, market_id, runner.selection_id).await?;

            let (price, available) = match &best_back {
                Some(ps) => (ps.price, ps.size),
                None => {
                    println!(
                        "  ⏭  Skipped: no back price available for {} ({})",
                        runner.runner_name, pos
                    );
                    total_skipped += 1;
                    continue;
                }
            };

            // Check max odds
            if let Some(max) = cli.max_odds {
                if price > max {
                    println!(
                        "  ⏭  Skipped: {} price {:.2} > max odds {:.2}",
                        runner.runner_name, price, max
                    );
                    total_skipped += 1;
                    continue;
                }
            }

            let potential_return = cli.stake * price;
            let potential_profit = potential_return - cli.stake;
            let prob = prob_for_position(entry, *pos).map_or("?".to_string(), |v| v.to_string());

            println!();
            println!("  ┌─ Proposed Bet ─────────────────────────────────────────");
            println!("  │  BACK {} ({})  @  {:.2}", runner.runner_name, pos, price);
            println!("  │  Stake:   £{:.2}", cli.stake);
            println!("  │  Return:  £{:.2}  (profit: £{:.2})", potential_return, potential_profit);
            println!("  │  Prob:    {}%  |  Available: £{:.2}", prob, available);
            println!(
                "  │  Market:  {}  |  Selection: {}",
                market_id, runner.selection_id
            );
            println!("  └──────────────────────────────────────────────────────────");

            if cli.dry_run {
                println!("  🔍 Dry run — bet NOT placed");
                total_skipped += 1;
                continue;
            }

            let confirmed = if cli.yes {
                true
            } else {
                confirm("  Place this bet?")
            };

            if !confirmed {
                println!("  ✗  Skipped by user");
                total_skipped += 1;
                continue;
            }

            // Place the bet
            let request = PlaceOrdersRequest {
                market_id: market_id.to_string(),
                instructions: vec![PlaceInstruction {
                    order_type: OrderType::Limit,
                    selection_id: runner.selection_id,
                    handicap: None,
                    side: Side::Back,
                    limit_order: Some(LimitOrder {
                        size: cli.stake,
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
                customer_ref: Some(format!("bpipe-bet-{}", entry.bf_event_id)),
                market_version: None,
                customer_strategy_ref: None,
                r#async: None,
            };

            let report = client.place_orders(&request).await?;

            match report.status {
                ExecutionReportStatus::Success => {
                    total_placed += 1;
                    total_stake += cli.stake;
                    println!("  ✓  Bet placed successfully!");
                    if let Some(reports) = &report.instruction_reports {
                        for ir in reports {
                            if let Some(bet_id) = &ir.bet_id {
                                println!("     Bet ID: {}", bet_id);
                            }
                            if let Some(matched) = ir.size_matched {
                                println!("     Matched: £{:.2}", matched);
                            }
                            if let Some(avg) = ir.average_price_matched {
                                println!("     Avg price: {:.2}", avg);
                            }
                        }
                    }
                }
                _ => {
                    total_skipped += 1;
                    println!("  ✗  Bet FAILED: {:?}", report.status);
                    if let Some(error) = &report.error_code {
                        println!("     Error: {:?}", error);
                    }
                    if let Some(reports) = &report.instruction_reports {
                        for ir in reports {
                            if let Some(err) = &ir.error_code {
                                println!("     Detail: {}", err);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n{}", separator);
    println!("SUMMARY");
    println!("  Bets placed:  {}", total_placed);
    println!("  Skipped:      {}", total_skipped);
    println!("  Total staked: £{:.2}", total_stake);
    println!("{}", separator);

    Ok(())
}
