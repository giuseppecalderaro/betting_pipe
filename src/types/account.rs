use serde::{Deserialize, Serialize};

/// Account funds response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountFundsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_to_bet_balance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_commission: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_balance: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
}

/// Account details response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetailsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_balance: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}
