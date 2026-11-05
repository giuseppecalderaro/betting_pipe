use serde::{Deserialize, Serialize};

// ─── Outgoing messages (client → Betfair) ───────────────

/// Authentication message sent after connection.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamAuthentication {
    pub op: String, // "authentication"
    pub id: i64,
    pub app_key: String,
    pub session: String,
}

impl std::fmt::Debug for StreamAuthentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamAuthentication")
            .field("op", &self.op)
            .field("id", &self.id)
            .field("app_key", &"<redacted>")
            .field("session", &"<redacted>")
            .finish()
    }
}

/// Market subscription request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSubscription {
    pub op: String, // "marketSubscription"
    pub id: i64,
    pub market_filter: StreamMarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_data_filter: Option<StreamMarketDataFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
}

/// Order subscription request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSubscription {
    pub op: String, // "orderSubscription"
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_filter: Option<OrderSubscriptionFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
}

/// Heartbeat message.
#[derive(Debug, Clone, Serialize)]
pub struct StreamHeartbeat {
    pub op: String, // "heartbeat"
    pub id: i64,
}

/// Filter for stream market subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamMarketFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betting_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venues: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_market: Option<bool>,
}

/// Filter for stream market data fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamMarketDataFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ladder_levels: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
}

/// Filter for order subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSubscriptionFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_overall_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_matched_by_strategy_ref: Option<bool>,
}

// ─── Incoming messages (Betfair → client) ───────────────

/// A generic stream message. We deserialize the `op` field first to determine the type.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum StreamMessage {
    /// Initial connection message.
    #[serde(rename = "connection")]
    Connection {
        #[serde(rename = "connectionId")]
        connection_id: String,
    },

    /// Status response (auth result, subscription result).
    #[serde(rename = "status")]
    Status {
        id: Option<i64>,
        #[serde(rename = "statusCode")]
        status_code: Option<String>,
        #[serde(rename = "connectionClosed")]
        connection_closed: Option<bool>,
        #[serde(rename = "errorCode")]
        error_code: Option<String>,
        #[serde(rename = "errorMessage")]
        error_message: Option<String>,
    },

    /// Market change message.
    #[serde(rename = "mcm")]
    MarketChange(MarketChangeMessage),

    /// Order change message.
    #[serde(rename = "ocm")]
    OrderChange(OrderChangeMessage),
}

/// Market change message payload.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChangeMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<String>, // "SUB_IMAGE", "RESUB_DELTA", "HEARTBEAT"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<i64>, // publish time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc: Option<Vec<MarketChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
}

/// A single market change.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChange {
    pub id: String, // market ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rc: Option<Vec<RunnerChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img: Option<bool>, // true = full image, false = delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv: Option<f64>, // total volume
    #[serde(rename = "marketDefinition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_definition: Option<serde_json::Value>, // complex, keep as Value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub con: Option<bool>, // conflated
}

/// A single runner change within a market change.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerChange {
    pub id: i64, // selection ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ltp: Option<f64>, // last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv: Option<f64>, // traded volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spn: Option<f64>, // starting price near
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spf: Option<f64>, // starting price far
    /// Best available to back: [[level, price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atb: Option<Vec<Vec<f64>>>,
    /// Best available to lay: [[level, price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atl: Option<Vec<Vec<f64>>>,
    /// Traded: [[price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trd: Option<Vec<Vec<f64>>>,
    /// Starting price back: [[price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spb: Option<Vec<Vec<f64>>>,
    /// Starting price lay: [[price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spl: Option<Vec<Vec<f64>>>,
    /// Best available to back (full depth): [[price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batb: Option<Vec<Vec<f64>>>,
    /// Best available to lay (full depth): [[price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batl: Option<Vec<Vec<f64>>>,
    /// Best display available to back: [[level, price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdatb: Option<Vec<Vec<f64>>>,
    /// Best display available to lay: [[level, price, size], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdatl: Option<Vec<Vec<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hc: Option<f64>, // handicap
}

/// Order change message payload.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderChangeMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oc: Option<Vec<OrderMarketChange>>,
}

/// Order changes for a specific market.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderMarketChange {
    pub id: String, // market ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orc: Option<Vec<OrderRunnerChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
}

/// Order changes for a runner within a market.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRunnerChange {
    pub id: i64, // selection ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uo: Option<Vec<StreamOrder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mb: Option<Vec<Vec<f64>>>, // matched backs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml: Option<Vec<Vec<f64>>>, // matched lays
}

/// An order in the stream.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamOrder {
    pub id: String, // bet ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<f64>, // price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<f64>, // size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>, // "B" or "L"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<String>, // persistence type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ot: Option<String>, // order type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "E" executable, "EC" execution complete
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sm: Option<f64>, // size matched
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sr: Option<f64>, // size remaining
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sl: Option<f64>, // size lapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sc: Option<f64>, // size cancelled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sv: Option<f64>, // size voided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avp: Option<f64>, // average price matched
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rc: Option<String>, // regulator code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfo: Option<String>, // customer order ref
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfs: Option<String>, // customer strategy ref
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pd: Option<i64>, // placed date (epoch ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md: Option<i64>, // matched date (epoch ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ld: Option<i64>, // lapsed date (epoch ms)
}
