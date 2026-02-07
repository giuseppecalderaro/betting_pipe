//! Scraper for statarea.com football predictions.

use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::{BpipeError, Result};

/// A scraped match prediction from statarea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedMatch {
    /// Competition name, e.g. "ICELAND - LEAGUE CUP GRP. 2"
    pub competition: String,
    /// Home team name
    pub home_team: String,
    /// Away team name
    pub away_team: String,
    /// Match datetime string, e.g. "2026-02-07 18:00"
    pub datetime: String,
    /// Statarea tip, e.g. "1", "X", "2", "1X", "X2", "12"
    pub tip: String,
    /// Win probability percentage (home)
    pub prob_home: Option<u32>,
    /// Draw probability percentage
    pub prob_draw: Option<u32>,
    /// Win probability percentage (away)
    pub prob_away: Option<u32>,
    /// Whether the match already has a result
    pub has_result: bool,
}

/// Scrape predictions from statarea.com.
pub async fn scrape_predictions(url: &str) -> Result<Vec<ScrapedMatch>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .map_err(|e| BpipeError::ApiError(format!("Failed to build HTTP client: {e}")))?;

    let html = client
        .get(url)
        .send()
        .await
        .map_err(|e| BpipeError::ApiError(format!("Failed to fetch {url}: {e}")))?
        .text()
        .await
        .map_err(|e| BpipeError::ApiError(format!("Failed to read response body: {e}")))?;

    debug!("Fetched {} bytes from {}", html.len(), url);

    parse_predictions(&html)
}

fn parse_predictions(html: &str) -> Result<Vec<ScrapedMatch>> {
    let doc = Html::parse_document(html);

    let competition_sel = Selector::parse("div.competition").unwrap();
    let comp_name_sel = Selector::parse("div.header div.name").unwrap();
    let match_sel = Selector::parse("div.match").unwrap();
    let host_name_sel = Selector::parse("div.hostteam div.name a").unwrap();
    let guest_name_sel = Selector::parse("div.guestteam div.name a").unwrap();
    let datetime_sel = Selector::parse("div.teams div.ownheader").unwrap();
    let tip_sel = Selector::parse("div.tip div.value div").unwrap();
    let coef_sel = Selector::parse("div.coefrow > div.coefbox > div.value").unwrap();
    let result_sel = Selector::parse("div.hostteam a.goals").unwrap();

    let mut matches = Vec::new();

    for comp_el in doc.select(&competition_sel) {
        // Get competition name
        let comp_name = comp_el
            .select(&comp_name_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Skip advertisement blocks
        if comp_name == "Advertisement" || comp_name.is_empty() {
            continue;
        }

        debug!("Competition: {}", comp_name);

        // Iterate over matches in this competition
        for match_el in comp_el.select(&match_sel) {
            // Home team
            let home_team = match match_el.select(&host_name_sel).next() {
                Some(el) => el.text().collect::<String>().trim().to_string(),
                None => continue,
            };

            // Away team
            let away_team = match match_el.select(&guest_name_sel).next() {
                Some(el) => el.text().collect::<String>().trim().to_string(),
                None => continue,
            };

            // Datetime from the ownheader text (skip marker div)
            let datetime = match_el
                .select(&datetime_sel)
                .next()
                .map(|el| {
                    el.text()
                        .collect::<String>()
                        .trim()
                        .replace('\u{a0}', " ")
                        .to_string()
                })
                .unwrap_or_default();

            // Tip
            let tip = match_el
                .select(&tip_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Check if match already has a result
            let has_result = match_el.select(&result_sel).next().is_some();

            // Prediction probabilities (1, X, 2 are the first 3 coefboxes)
            let coefs: Vec<u32> = match_el
                .select(&coef_sel)
                .take(3)
                .filter_map(|el| {
                    el.text()
                        .collect::<String>()
                        .trim()
                        .parse::<u32>()
                        .ok()
                })
                .collect();

            let (prob_home, prob_draw, prob_away) = if coefs.len() == 3 {
                (Some(coefs[0]), Some(coefs[1]), Some(coefs[2]))
            } else {
                (None, None, None)
            };

            matches.push(ScrapedMatch {
                competition: comp_name.clone(),
                home_team,
                away_team,
                datetime,
                tip,
                prob_home,
                prob_draw,
                prob_away,
                has_result,
            });
        }
    }

    Ok(matches)
}
