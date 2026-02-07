use serde::de::DeserializeOwned;
use tracing::debug;

use crate::auth::Session;
use crate::config::BetfairConfig;
use crate::error::{BpipeError, Result};

/// HTTP client for Betfair REST API calls.
#[derive(Debug)]
pub struct BetfairClient {
    http: reqwest::Client,
    session: Session,
    betting_url: String,
    account_url: String,
    identity_url: String,
}

impl BetfairClient {
    /// Create a new client with an active session.
    pub fn new(session: Session, config: &BetfairConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            session,
            betting_url: config.urls.betting_api.clone(),
            account_url: config.urls.account_api.clone(),
            identity_url: config.urls.identity_sso.clone(),
        }
    }

    /// Get a reference to the current session.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Send a POST request to a Betfair betting API endpoint.
    pub async fn betting_request<Req, Resp>(&self, operation: &str, body: &Req) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let url = format!("{}/{}/", self.betting_url, operation);
        self.post_json(&url, body).await
    }

    /// Send a POST request to a Betfair account API endpoint.
    pub async fn account_request<Req, Resp>(&self, operation: &str, body: &Req) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let url = format!("{}/{}/", self.account_url, operation);
        self.post_json(&url, body).await
    }

    /// Send a GET request to a Betfair account API endpoint (no body).
    pub async fn account_get<Resp>(&self, operation: &str) -> Result<Resp>
    where
        Resp: DeserializeOwned,
    {
        let url = format!("{}/{}/", self.account_url, operation);
        debug!("GET {}", url);

        let resp = self
            .http
            .get(&url)
            .header("X-Application", &self.session.app_key)
            .header("X-Authentication", &self.session.session_token)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;
        debug!("Response status={}, len={}", status, body.len());

        if !status.is_success() {
            return Err(BpipeError::ApiError(format!(
                "HTTP {} — {}",
                status, body
            )));
        }

        self.parse_response(&body)
    }

    /// Keep the session alive.
    pub async fn keep_alive(&self) -> Result<()> {
        crate::auth::keep_alive(&self.session, &self.identity_url).await
    }

    /// Internal: POST JSON and deserialize the response.
    async fn post_json<Req, Resp>(&self, url: &str, body: &Req) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let json_body = serde_json::to_string(body)?;
        debug!("POST {} body={}", url, json_body);

        let resp = self
            .http
            .post(url)
            .header("X-Application", &self.session.app_key)
            .header("X-Authentication", &self.session.session_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json_body)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;
        debug!("Response status={}, len={}", status, body.len());

        if !status.is_success() {
            return Err(BpipeError::ApiError(format!(
                "HTTP {} — {}",
                status, body
            )));
        }

        self.parse_response(&body)
    }

    /// Parse a JSON response, handling Betfair error format.
    fn parse_response<T: DeserializeOwned>(&self, body: &str) -> Result<T> {
        // Check for APINGException error format
        if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(body) {
            if let Some(fault_code) = err_val.get("faultcode").and_then(|v| v.as_str()) {
                let fault_string = err_val
                    .get("faultstring")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                let detail = err_val
                    .get("detail")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                return Err(BpipeError::ApingException {
                    fault_code: fault_code.to_string(),
                    fault_string,
                    detail,
                });
            }
        }

        serde_json::from_str(body).map_err(|e| {
            BpipeError::JsonError(e)
        })
    }
}
