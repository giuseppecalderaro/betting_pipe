use crate::client::BetfairClient;
use crate::error::Result;
use crate::types::account::*;

/// Account API operations.
impl BetfairClient {
    /// Get account funds (balance, exposure).
    pub async fn get_account_funds(&self) -> Result<AccountFundsResponse> {
        // The account endpoint accepts an empty JSON body or no body
        self.account_request("getAccountFunds", &serde_json::json!({}))
            .await
    }

    /// Get account details (name, currency, etc.).
    pub async fn get_account_details(&self) -> Result<AccountDetailsResponse> {
        self.account_request("getAccountDetails", &serde_json::json!({}))
            .await
    }
}
