use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Enums ───────────────────────────────────────────────

/// Side of a bet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Back,
    Lay,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Back => write!(f, "BACK"),
            Side::Lay => write!(f, "LAY"),
        }
    }
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    LimitOnClose,
    MarketOnClose,
}

/// Persistence type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceType {
    Lapse,
    Persist,
    MarketOnClose,
}

/// Time in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    FillOrKill,
}

/// Bet target type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BetTargetType {
    BackersProfit,
    Payout,
}

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketStatus {
    Inactive,
    Open,
    Suspended,
    Closed,
}

/// Runner status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerStatus {
    Active,
    Winner,
    Loser,
    Removed,
    #[serde(rename = "REMOVED_VACANT")]
    RemovedVacant,
    Hidden,
    #[serde(rename = "PLACED")]
    Placed,
}

/// Market betting type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketBettingType {
    Odds,
    Line,
    Range,
    AsianHandicapDoubleLine,
    AsianHandicapSingleLine,
    FixedOdds,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Pending,
    ExecutionComplete,
    Executable,
    Expired,
}

/// Sort direction for order queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortDir {
    EarliestToLatest,
    LatestToEarliest,
}

/// Order projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProjection {
    All,
    Executable,
    ExecutionComplete,
}

/// Market sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketSort {
    MinimumTraded,
    MaximumTraded,
    MinimumAvailable,
    MaximumAvailable,
    FirstToStart,
    LastToStart,
}

/// Price projection data fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceData {
    SpAvailable,
    SpTraded,
    ExBestOffers,
    ExAllOffers,
    ExTraded,
}

/// Match projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchProjection {
    NoRollup,
    RolledUpByPrice,
    RolledUpByAvgPrice,
}

/// Market projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketProjection {
    Competition,
    Event,
    EventType,
    MarketStartTime,
    MarketDescription,
    RunnerDescription,
    RunnerMetadata,
}

/// Execution report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionReportStatus {
    Success,
    Failure,
    ProcessedWithErrors,
    Timeout,
}

/// Execution report error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionReportErrorCode {
    ErrorInMatcher,
    ProcessedWithErrors,
    BetActionError,
    InvalidAccountState,
    InvalidWalletStatus,
    InsufficientFunds,
    LossLimitExceeded,
    MarketSuspended,
    MarketNotOpenForBetting,
    DuplicateTransaction,
    InvalidOrder,
    InvalidMarketId,
    PermissionDenied,
    DuplicateBetids,
    NoActionRequired,
    ServiceUnavailable,
    RejectedByRegulator,
    NoChasing,
    RegulatorIsNotAvailable,
    TooManyInstructions,
    InvalidMarketVersion,
    EventExposureLimitExceeded,
    EventMatchedExposureLimitExceeded,
    EventBlockedExposureLimitExceeded,
}

/// Instruction report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstructionReportStatus {
    Success,
    Failure,
    Timeout,
}

// ─── Common structures ──────────────────────────────────

/// A time range filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<DateTime<Utc>>,
}

/// A price and size pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSize {
    pub price: f64,
    pub size: f64,
}

/// Filter for markets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venues: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_play_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_betting_types: Option<Vec<MarketBettingType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_countries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_orders: Option<Vec<OrderStatus>>,
}

/// Price projection for market book queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceProjection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_data: Option<Vec<PriceData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex_best_offers_overrides: Option<ExBestOffersOverrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtualise: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollover_limit: Option<i32>,
}

/// Overrides for best offers depth.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExBestOffersOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_prices_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabk_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_limit: Option<i32>,
}
