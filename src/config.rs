use std::path::PathBuf;

/// Configuration for the Betfair API client.
#[derive(Debug, Clone)]
pub struct BetfairConfig {
    /// Application key (required for all API calls).
    pub app_key: String,

    /// Authentication method.
    pub auth: AuthConfig,

    /// Locale for API responses (e.g., "en").
    pub locale: Option<String>,

    /// Base URLs (defaults are provided).
    pub urls: ApiUrls,
}

/// Authentication configuration.
#[derive(Debug, Clone)]
pub enum AuthConfig {
    /// Interactive login with username and password.
    Interactive {
        username: String,
        password: String,
    },
    /// Certificate-based (bot) login.
    Certificate {
        username: String,
        password: String,
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

/// API endpoint URLs.
#[derive(Debug, Clone)]
pub struct ApiUrls {
    pub identity_sso: String,
    pub identity_cert: String,
    pub betting_api: String,
    pub account_api: String,
    pub stream_api: String,
    pub stream_port: u16,
}

impl Default for ApiUrls {
    fn default() -> Self {
        Self {
            identity_sso: "https://identitysso.betfair.com/api".into(),
            identity_cert: "https://identitysso-cert.betfair.com/api".into(),
            betting_api: "https://api.betfair.com/exchange/betting/rest/v1.0".into(),
            account_api: "https://api.betfair.com/exchange/account/rest/v1.0".into(),
            stream_api: "stream-api.betfair.com".into(),
            stream_port: 443,
        }
    }
}

impl BetfairConfig {
    /// Create a config for interactive login.
    pub fn interactive(app_key: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            app_key: app_key.into(),
            auth: AuthConfig::Interactive {
                username: username.into(),
                password: password.into(),
            },
            locale: None,
            urls: ApiUrls::default(),
        }
    }

    /// Create a config for certificate-based login.
    pub fn certificate(
        app_key: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        cert_path: impl Into<PathBuf>,
        key_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            app_key: app_key.into(),
            auth: AuthConfig::Certificate {
                username: username.into(),
                password: password.into(),
                cert_path: cert_path.into(),
                key_path: key_path.into(),
            },
            locale: None,
            urls: ApiUrls::default(),
        }
    }
}
