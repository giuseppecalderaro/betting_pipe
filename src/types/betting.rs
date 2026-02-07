use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::*;

// ─── Response types ─────────────────────────────────────

/// An event type (sport).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTypeResult {
    pub event_type: EventType,
    pub market_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventType {
    pub id: String,
    pub name: String,
}

/// A competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionResult {
    pub competition: Competition,
    pub market_count: Option<i32>,
    pub competition_region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Competition {
    pub id: String,
    pub name: String,
}

/// An event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResult {
    pub event: Event,
    pub market_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_date: Option<DateTime<Utc>>,
}

/// Market type result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTypeResult {
    pub market_type: String,
    pub market_count: Option<i32>,
}

/// Country result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountryCodeResult {
    pub country_code: String,
    pub market_count: Option<i32>,
}

/// Venue result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueResult {
    pub venue: String,
    pub market_count: Option<i32>,
}

// ─── Market Catalogue ───────────────────────────────────

/// Detailed market information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCatalogue {
    pub market_id: String,
    pub market_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MarketDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<RunnerCatalog>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<Competition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Event>,
}

/// Market description.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDescription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_market: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settle_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betting_type: Option<MarketBettingType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_base_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_allowed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_has_date: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each_way_divisor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarifications: Option<String>,
}

/// Runner in a market catalogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerCatalog {
    pub selection_id: i64,
    pub runner_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handicap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

// ─── Market Book (live prices) ──────────────────────────

/// Live market data (prices, depth).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBook {
    pub market_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_market_data_delayed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_delay: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_reconciled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inplay: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_winners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_runners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_active_runners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_match_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_available: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_matching: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners_voidable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<Runner>>,
}

/// A runner in a market book (live prices).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub selection_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handicap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RunnerStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price_traded: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sp: Option<StartingPrices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex: Option<ExchangePrices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<Match>>,
}

/// Starting prices data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingPrices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub near_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub far_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_stakeholders: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lay_liability_stakeholders: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_sp: Option<f64>,
}

/// Exchange prices (back/lay).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangePrices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_to_back: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_to_lay: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traded_volume: Option<Vec<PriceSize>>,
}

/// An order on a runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub bet_id: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub persistence_type: PersistenceType,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub bsp_liability: f64,
    pub placed_date: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_lapsed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_voided: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

/// A matched portion of a bet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_id: Option<String>,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_date: Option<DateTime<Utc>>,
}

// ─── Place / Cancel / Replace ───────────────────────────

/// Instruction to place a bet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInstruction {
    pub order_type: OrderType,
    pub selection_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handicap: Option<f64>,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_order: Option<LimitOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_on_close_order: Option<LimitOnCloseOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_on_close_order: Option<MarketOnCloseOrder>,
}

/// A limit order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrder {
    pub size: f64,
    pub price: f64,
    pub persistence_type: PersistenceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fill_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_target_type: Option<BetTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_target_size: Option<f64>,
}

/// A limit on close order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOnCloseOrder {
    pub liability: f64,
    pub price: f64,
}

/// A market on close order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOnCloseOrder {
    pub liability: f64,
}

/// Instruction to cancel a bet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInstruction {
    pub bet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_reduction: Option<f64>,
}

/// Instruction to replace a bet (new price).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceInstruction {
    pub bet_id: String,
    pub new_price: f64,
}

/// Instruction to update a bet (persistence).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstruction {
    pub bet_id: String,
    pub new_persistence_type: PersistenceType,
}

// ─── Execution reports ──────────────────────────────────

/// Response from placing orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceExecutionReport {
    pub status: ExecutionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ExecutionReportErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<PlaceInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Report for a single placed order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInstructionReport {
    pub status: InstructionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_status: Option<OrderStatus>,
    pub instruction: PlaceInstruction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
}

/// Response from cancelling orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelExecutionReport {
    pub status: ExecutionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ExecutionReportErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<CancelInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Report for a single cancelled order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInstructionReport {
    pub status: InstructionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<CancelInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_date: Option<DateTime<Utc>>,
}

/// Response from replacing orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceExecutionReport {
    pub status: ExecutionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ExecutionReportErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<ReplaceInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Report for a single replaced order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceInstructionReport {
    pub status: InstructionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_instruction_report: Option<CancelInstructionReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_instruction_report: Option<PlaceInstructionReport>,
}

/// Response from updating orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExecutionReport {
    pub status: ExecutionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ExecutionReportErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<UpdateInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Report for a single updated order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstructionReport {
    pub status: InstructionReportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub instruction: UpdateInstruction,
}

// ─── Current / Cleared Orders ───────────────────────────

/// Summary of a current (live) order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentOrderSummary {
    pub bet_id: String,
    pub market_id: String,
    pub selection_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handicap: Option<f64>,
    pub price_size: PriceSize,
    pub bsp_liability: f64,
    pub side: Side,
    pub status: OrderStatus,
    pub persistence_type: PersistenceType,
    pub order_type: OrderType,
    pub placed_date: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_lapsed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_voided: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_auth_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

/// Paginated list of current orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentOrderSummaryReport {
    pub current_orders: Vec<CurrentOrderSummary>,
    pub more_available: bool,
}

// ─── Request bodies ─────────────────────────────────────

/// Request body for placeOrders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrdersRequest {
    pub market_id: String,
    pub instructions: Vec<PlaceInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
}

/// Request body for cancelOrders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<CancelInstruction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Request body for replaceOrders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceOrdersRequest {
    pub market_id: String,
    pub instructions: Vec<ReplaceInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
}

/// Request body for updateOrders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrdersRequest {
    pub market_id: String,
    pub instructions: Vec<UpdateInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

/// Request body for listMarketCatalogue.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMarketCatalogueRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_projection: Option<Vec<MarketProjection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<MarketSort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

/// Request body for listMarketBook.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMarketBookRequest {
    pub market_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_projection: Option<PriceProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_projection: Option<OrderProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_projection: Option<MatchProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

/// Request body for listCurrentOrders.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCurrentOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_projection: Option<OrderProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<SortDir>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_record: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<i32>,
}

/// Generic filter request body.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}
