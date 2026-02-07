use serde::Deserialize;
use tracing::{info, debug};

use crate::config::{AuthConfig, BetfairConfig};
use crate::error::{BpipeError, Result};

/// Response from Betfair login endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    session_token: Option<String>,
    login_status: Option<String>,
    token: Option<String>,  // cert login uses "token" instead
    status: Option<String>, // cert login uses "status"
    error: Option<String>,
}

/// Active session holding the token and app key.
#[derive(Debug, Clone)]
pub struct Session {
    pub session_token: String,
    pub app_key: String,
}

/// Perform authentication based on the provided config.
pub async fn login(config: &BetfairConfig) -> Result<Session> {
    match &config.auth {
        AuthConfig::Interactive { username, password } => {
            interactive_login(&config.app_key, username, password, &config.urls.identity_sso).await
        }
        AuthConfig::Certificate {
            username,
            password,
            cert_path,
            key_path,
        } => {
            cert_login(
                &config.app_key,
                username,
                password,
                cert_path,
                key_path,
                &config.urls.identity_cert,
            )
            .await
        }
    }
}

/// Interactive login with username/password.
async fn interactive_login(
    app_key: &str,
    username: &str,
    password: &str,
    base_url: &str,
) -> Result<Session> {
    let url = format!("{}/login", base_url);
    debug!("Interactive login to {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("X-Application", app_key)
        .header("Accept", "application/json")
        .form(&[("username", username), ("password", password)])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    debug!("Login response status={}, body={}", status, body);

    if !status.is_success() {
        return Err(BpipeError::AuthError(format!(
            "HTTP {} — {}",
            status, body
        )));
    }

    let login_resp: LoginResponse = serde_json::from_str(&body)?;

    // The interactive endpoint returns sessionToken + loginStatus
    let token = login_resp
        .session_token
        .or(login_resp.token)
        .ok_or_else(|| BpipeError::AuthError("No session token in response".into()))?;

    let login_status = login_resp
        .login_status
        .or(login_resp.status)
        .unwrap_or_default();

    if login_status != "SUCCESS" {
        return Err(BpipeError::AuthError(format!(
            "Login failed: status={}, error={:?}",
            login_status, login_resp.error
        )));
    }

    info!("Successfully authenticated (interactive)");
    Ok(Session {
        session_token: token,
        app_key: app_key.to_string(),
    })
}

/// Certificate-based (bot) login.
async fn cert_login(
    app_key: &str,
    username: &str,
    password: &str,
    cert_path: &std::path::Path,
    key_path: &std::path::Path,
    base_url: &str,
) -> Result<Session> {
    let url = format!("{}/certlogin", base_url);
    debug!("Certificate login to {}", url);

    // Read the client certificate and key
    let cert_pem = std::fs::read(cert_path).map_err(|e| {
        BpipeError::ConfigError(format!("Failed to read certificate file {:?}: {}", cert_path, e))
    })?;
    let key_pem = std::fs::read(key_path).map_err(|e| {
        BpipeError::ConfigError(format!("Failed to read key file {:?}: {}", key_path, e))
    })?;

    // reqwest with native-tls expects Identity from pkcs8 pem (cert + key)
    let identity = reqwest::Identity::from_pkcs8_pem(&cert_pem, &key_pem)?;

    let client = reqwest::Client::builder()
        .identity(identity)
        .build()?;

    let resp = client
        .post(&url)
        .header("X-Application", app_key)
        .header("Accept", "application/json")
        .form(&[("username", username), ("password", password)])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    debug!("Cert login response status={}, body={}", status, body);

    if !status.is_success() {
        return Err(BpipeError::AuthError(format!(
            "HTTP {} — {}",
            status, body
        )));
    }

    let login_resp: LoginResponse = serde_json::from_str(&body)?;

    let token = login_resp
        .session_token
        .or(login_resp.token)
        .ok_or_else(|| BpipeError::AuthError("No session token in response".into()))?;

    let login_status = login_resp
        .login_status
        .or(login_resp.status)
        .unwrap_or_default();

    if login_status != "SUCCESS" {
        return Err(BpipeError::AuthError(format!(
            "Login failed: status={}, error={:?}",
            login_status, login_resp.error
        )));
    }

    info!("Successfully authenticated (certificate)");
    Ok(Session {
        session_token: token,
        app_key: app_key.to_string(),
    })
}

/// Keep-alive: extend the session.
pub async fn keep_alive(session: &Session, base_url: &str) -> Result<()> {
    let url = format!("{}/keepAlive", base_url);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("X-Application", &session.app_key)
        .header("X-Authentication", &session.session_token)
        .header("Accept", "application/json")
        .send()
        .await?;

    let body = resp.text().await?;
    let login_resp: LoginResponse = serde_json::from_str(&body)?;
    let status = login_resp
        .login_status
        .or(login_resp.status)
        .unwrap_or_default();

    if status != "SUCCESS" {
        return Err(BpipeError::AuthError(format!(
            "Keep-alive failed: {}",
            status
        )));
    }

    debug!("Session kept alive");
    Ok(())
}

/// Logout and invalidate the session.
pub async fn logout(session: &Session, base_url: &str) -> Result<()> {
    let url = format!("{}/logout", base_url);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("X-Application", &session.app_key)
        .header("X-Authentication", &session.session_token)
        .header("Accept", "application/json")
        .send()
        .await?;

    let body = resp.text().await?;
    debug!("Logout response: {}", body);
    info!("Logged out successfully");
    Ok(())
}
