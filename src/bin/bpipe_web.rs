use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};

// ── CLI ──────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "bpipe-web", about = "Web server for matched file viewer")]
struct Cli {
    /// Directory containing matched_*.txt files (defaults to cwd)
    #[arg(long, default_value = ".")]
    data_dir: PathBuf,

    /// Directory containing built web frontend (defaults to web/dist)
    #[arg(long, default_value = "web/dist")]
    static_dir: PathBuf,

    /// Port to listen on
    #[arg(long, default_value_t = 3001)]
    port: u16,
}

// ── Types ────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct MatchEntry {
    has_result: bool,
    match_name: String,
    date: String,
    tip: String,
    prob_home: Option<u32>,
    prob_draw: Option<u32>,
    prob_away: Option<u32>,
    bf_event_id: String,
    bf_event_name: String,
}

#[derive(Serialize)]
struct MatchesResponse {
    file_name: String,
    generated_at: String,
    total_scraped: u32,
    total_matched: u32,
    total_unmatched: u32,
    matches: Vec<MatchEntry>,
}

#[derive(Serialize)]
struct FileListResponse {
    files: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

struct AppState {
    data_dir: PathBuf,
}

// ── Parser ───────────────────────────────────────────────────────────

fn parse_matched_file(content: &str) -> Result<MatchesResponse> {
    let lines: Vec<&str> = content.lines().collect();
    let mut matches = Vec::new();
    let mut total_scraped: u32 = 0;
    let mut total_matched: u32 = 0;
    let mut total_unmatched: u32 = 0;
    let generated_at = String::new();

    for line in &lines {
        // Skip header and separator lines
        if line.starts_with("MATCH") || line.starts_with('─') || line.trim().is_empty() {
            continue;
        }

        // Parse the summary line at the bottom
        if line.contains("matched to Betfair") {
            // "185 matched to Betfair, 181 unmatched (out of 366 scraped)"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                total_matched = parts[0].parse().unwrap_or(0);
                total_unmatched = parts[4].parse().unwrap_or(0);
                // extract number from "(out" ... "of" ... "366" ... "scraped)"
                if let Some(pos) = parts.iter().position(|&p| p == "of") {
                    if pos + 1 < parts.len() {
                        total_scraped = parts[pos + 1]
                            .trim_end_matches(')')
                            .parse()
                            .unwrap_or(0);
                    }
                }
            }
            continue;
        }

        // Parse a match line using char-based indexing (handles multi-byte UTF-8)
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 80 {
            continue;
        }

        // Check for result marker
        let has_result = chars[0] == '✓';

        // The format is fixed-width columns. Parse by finding field boundaries.
        // We need to extract fields from the fixed-width format.
        // Strategy: work with the string content directly
        let entry = parse_match_line(line, has_result);
        if let Some(entry) = entry {
            matches.push(entry);
        }
    }

    total_matched = if total_matched > 0 {
        total_matched
    } else {
        matches.len() as u32
    };

    Ok(MatchesResponse {
        file_name: String::new(),
        generated_at,
        total_scraped,
        total_matched,
        total_unmatched,
        matches,
    })
}

fn parse_match_line(line: &str, has_result: bool) -> Option<MatchEntry> {
    // The matched file has a fixed-width format. We'll parse by looking for
    // the date pattern (YYYY-MM-DD HH:MM) as an anchor point.
    let chars: Vec<char> = line.chars().collect();

    // Find the date pattern: 4 digits, dash, 2 digits, dash, 2 digits, space, 2 digits, colon, 2 digits
    let date_start = find_date_position(&chars)?;

    // Match name is everything from position 1 (skip ✓ or space) to date_start, trimmed
    let start = if has_result { 1 } else { 1 };
    let match_name: String = chars[start..date_start].iter().collect::<String>().trim().to_string();

    // Date is 16 chars: "2026-02-08 00:00"
    let date: String = chars[date_start..date_start + 16].iter().collect();

    // After date, skip spaces to find tip
    let after_date = date_start + 16;
    let remaining: String = chars[after_date..].iter().collect();
    let remaining = remaining.trim_start();

    // Parse: TIP  1%  X%  2%  BF_EVENT_ID  BF_EVENT_NAME
    let tokens: Vec<&str> = remaining.split_whitespace().collect();

    if tokens.len() < 5 {
        return None;
    }

    let tip = tokens[0].to_string();
    let prob_home: Option<u32> = tokens[1].parse().ok();
    let prob_draw: Option<u32> = tokens[2].parse().ok();
    let prob_away: Option<u32> = tokens[3].parse().ok();
    let bf_event_id = tokens[4].to_string();
    let bf_event_name = if tokens.len() > 5 {
        tokens[5..].join(" ")
    } else {
        String::new()
    };

    // Clean up bf_event_name (may end with trailing spaces)
    let bf_event_name = bf_event_name.trim().to_string();

    Some(MatchEntry {
        has_result,
        match_name,
        date: date.trim().to_string(),
        tip,
        prob_home,
        prob_draw,
        prob_away,
        bf_event_id,
        bf_event_name,
    })
}

fn find_date_position(chars: &[char]) -> Option<usize> {
    // Look for pattern: digit digit digit digit '-' digit digit '-' digit digit ' ' digit digit ':' digit digit
    if chars.len() < 16 {
        return None;
    }
    for i in 0..chars.len() - 15 {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
            && chars[i + 4] == '-'
            && chars[i + 5].is_ascii_digit()
            && chars[i + 6].is_ascii_digit()
            && chars[i + 7] == '-'
            && chars[i + 8].is_ascii_digit()
            && chars[i + 9].is_ascii_digit()
            && chars[i + 10] == ' '
            && chars[i + 11].is_ascii_digit()
            && chars[i + 12].is_ascii_digit()
            && chars[i + 13] == ':'
            && chars[i + 14].is_ascii_digit()
            && chars[i + 15].is_ascii_digit()
        {
            return Some(i);
        }
    }
    None
}

// ── Handlers ─────────────────────────────────────────────────────────

fn find_matched_files(data_dir: &PathBuf) -> Result<Vec<String>> {
    let mut files: Vec<String> = std::fs::read_dir(data_dir)
        .context("Failed to read data directory")?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("matched_") && name.ends_with(".txt") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files.reverse(); // newest first
    Ok(files)
}

async fn get_files(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FileListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let files = find_matched_files(&state.data_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(Json(FileListResponse { files }))
}

async fn get_matches(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<MatchesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let files = find_matched_files(&state.data_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let file_name = if let Some(name) = params.get("file") {
        // Validate that the requested file is in the list (prevent path traversal)
        if !files.contains(name) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("File not found: {}", name),
                }),
            ));
        }
        name.clone()
    } else {
        files.first().cloned().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "No matched files found".to_string(),
                }),
            )
        })?
    };

    let path = state.data_dir.join(&file_name);
    let content = std::fs::read_to_string(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to read {}: {}", file_name, e),
            }),
        )
    })?;

    let mut response = parse_matched_file(&content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse {}: {}", file_name, e),
            }),
        )
    })?;

    // Extract timestamp from filename: matched_20260208_084032.txt
    if file_name.len() >= 24 {
        let ts = &file_name[8..23]; // "20260208_084032"
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y%m%d_%H%M%S") {
            response.generated_at = dt.format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }
    response.file_name = file_name;

    Ok(Json(response))
}

// ── Main ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let state = Arc::new(AppState {
        data_dir: cli.data_dir.clone(),
    });

    let api = Router::new()
        .route("/api/files", get(get_files))
        .route("/api/matches", get(get_matches))
        .with_state(state);

    let app = if cli.static_dir.exists() {
        api.fallback_service(ServeDir::new(&cli.static_dir))
    } else {
        eprintln!(
            "⚠  Static dir {:?} not found — serving API only. Build the frontend with: cd web && npm run build",
            cli.static_dir
        );
        api.fallback_service(tower_http::services::ServeDir::new("."))
    };

    let app = app.layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("🌐 bpipe-web listening on http://localhost:{}", cli.port);
    println!("   Data dir: {:?}", cli.data_dir);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
