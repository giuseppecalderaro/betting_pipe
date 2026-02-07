use thiserror::Error;

/// Top-level error type for the bpipe library.
#[derive(Debug, Error)]
pub enum BpipeError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Streaming error: {0}")]
    StreamError(String),

    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Session expired or not authenticated")]
    NotAuthenticated,

    #[error("Betfair APINGException: {fault_code} — {fault_string}")]
    ApingException {
        fault_code: String,
        fault_string: String,
        detail: Option<BetfairErrorDetail>,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Detailed error information from the Betfair API.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetfairErrorDetail {
    pub exception_name: Option<String>,
    #[serde(rename = "APINGException")]
    pub aping_exception: Option<ApingExceptionDetail>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApingExceptionDetail {
    pub error_code: Option<String>,
    pub error_details: Option<String>,
    pub request_uuid: Option<String>,
}

pub type Result<T> = std::result::Result<T, BpipeError>;
