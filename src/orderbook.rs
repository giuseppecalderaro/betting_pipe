use serde::Serialize;

use crate::types::streaming::{MarketChange, RunnerChange};

/// Maintained order book state for a market, built from streaming deltas.
#[derive(Debug, Clone, Serialize)]
pub struct OrderBook {
    pub market_id: String,
    pub status: Option<String>,
    pub inplay: Option<bool>,
    pub total_volume: f64,
    pub runners: Vec<RunnerBook>,
    /// Epoch ms of last update.
    pub last_update: Option<i64>,
}

/// Order book for a single runner (selection).
#[derive(Debug, Clone, Serialize)]
pub struct RunnerBook {
    pub selection_id: i64,
    pub last_traded_price: Option<f64>,
    pub traded_volume: Option<f64>,
    /// Back prices: price → size (sorted descending = best back first).
    pub back: Vec<PriceLevel>,
    /// Lay prices: price → size (sorted ascending = best lay first).
    pub lay: Vec<PriceLevel>,
}

/// A single price level in the order book.
#[derive(Debug, Clone, Serialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

impl OrderBook {
    pub fn new(market_id: String) -> Self {
        Self {
            market_id,
            status: None,
            inplay: None,
            total_volume: 0.0,
            runners: Vec::new(),
            last_update: None,
        }
    }

    /// Apply a market change (delta or full image) to this order book.
    pub fn apply_change(&mut self, change: &MarketChange, publish_time: Option<i64>) {
        if let Some(tv) = change.tv {
            self.total_volume = tv;
        }

        self.last_update = publish_time.or(self.last_update);

        // Parse market definition for status/inplay if present
        if let Some(def) = &change.market_definition {
            if let Some(status) = def.get("status").and_then(|v| v.as_str()) {
                self.status = Some(status.to_string());
            }
            if let Some(inplay) = def.get("inPlay").and_then(|v| v.as_bool()) {
                self.inplay = Some(inplay);
            }
        }

        let is_image = change.img.unwrap_or(false);

        if is_image {
            // Full image: replace all runners
            self.runners.clear();
        }

        if let Some(runner_changes) = &change.rc {
            for rc in runner_changes {
                self.apply_runner_change(rc, is_image);
            }
        }
    }

    fn apply_runner_change(&mut self, rc: &RunnerChange, is_image: bool) {
        // Find or create the runner
        let runner = if let Some(r) = self.runners.iter_mut().find(|r| r.selection_id == rc.id) {
            r
        } else {
            self.runners.push(RunnerBook {
                selection_id: rc.id,
                last_traded_price: None,
                traded_volume: None,
                back: Vec::new(),
                lay: Vec::new(),
            });
            self.runners.last_mut().unwrap()
        };

        if let Some(ltp) = rc.ltp {
            runner.last_traded_price = Some(ltp);
        }
        if let Some(tv) = rc.tv {
            runner.traded_volume = Some(tv);
        }

        // Apply back prices
        // Betfair sends atb as [[price, size], ...] for level-less format
        // or batb as [[level, price, size], ...] for levelled format
        if let Some(atb) = &rc.atb {
            if is_image {
                runner.back.clear();
            }
            apply_price_levels(&mut runner.back, atb, false);
            runner.back.sort_by(|a, b| b.price.total_cmp(&a.price));
        }
        if let Some(batb) = &rc.batb {
            if is_image {
                runner.back.clear();
            }
            apply_levelled_prices(&mut runner.back, batb);
            runner.back.sort_by(|a, b| b.price.total_cmp(&a.price));
        }
        if let Some(bdatb) = &rc.bdatb {
            if is_image {
                runner.back.clear();
            }
            apply_levelled_prices(&mut runner.back, bdatb);
            runner.back.sort_by(|a, b| b.price.total_cmp(&a.price));
        }

        // Apply lay prices
        if let Some(atl) = &rc.atl {
            if is_image {
                runner.lay.clear();
            }
            apply_price_levels(&mut runner.lay, atl, false);
            runner.lay.sort_by(|a, b| a.price.total_cmp(&b.price));
        }
        if let Some(batl) = &rc.batl {
            if is_image {
                runner.lay.clear();
            }
            apply_levelled_prices(&mut runner.lay, batl);
            runner.lay.sort_by(|a, b| a.price.total_cmp(&b.price));
        }
        if let Some(bdatl) = &rc.bdatl {
            if is_image {
                runner.lay.clear();
            }
            apply_levelled_prices(&mut runner.lay, bdatl);
            runner.lay.sort_by(|a, b| a.price.total_cmp(&b.price));
        }

        // Remove zero-size levels
        runner.back.retain(|l| l.size > 0.0);
        runner.lay.retain(|l| l.size > 0.0);
    }
}

/// Apply [[price, size], ...] updates. Upserts at each price level.
fn apply_price_levels(book: &mut Vec<PriceLevel>, data: &[Vec<f64>], _levelled: bool) {
    for entry in data {
        if entry.len() >= 2 {
            let price = entry[0];
            let size = entry[1];
            if let Some(existing) = book.iter_mut().find(|l| (l.price - price).abs() < 1e-10) {
                existing.size = size;
            } else {
                book.push(PriceLevel { price, size });
            }
        }
    }
}

/// Apply [[level, price, size], ...] updates. Upserts at each price level.
fn apply_levelled_prices(book: &mut Vec<PriceLevel>, data: &[Vec<f64>]) {
    for entry in data {
        if entry.len() >= 3 {
            let price = entry[1];
            let size = entry[2];
            if let Some(existing) = book.iter_mut().find(|l| (l.price - price).abs() < 1e-10) {
                existing.size = size;
            } else {
                book.push(PriceLevel { price, size });
            }
        }
    }
}
