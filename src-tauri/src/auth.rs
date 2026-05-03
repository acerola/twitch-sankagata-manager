use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type RefreshGuard = Arc<tokio::sync::Mutex<()>>;

pub fn make_refresh_guard() -> RefreshGuard {
    Arc::new(tokio::sync::Mutex::new(()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub obtained_at: u64,
}

impl StoredTokens {
    pub fn stamp_now(mut self) -> Self {
        self.obtained_at = now_secs();
        self
    }

    pub fn is_fresh(&self, skew_secs: u64) -> bool {
        if self.obtained_at == 0 {
            return false;
        }
        self.obtained_at.saturating_add(self.expires_in) > now_secs().saturating_add(skew_secs)
    }
}

#[derive(Clone)]
pub struct DeviceFlow {
    base: String,
    client_id: String,
    scopes: Vec<String>,
    http: Client,
}

impl DeviceFlow {
    pub fn new(base: impl Into<String>, client_id: impl Into<String>, scopes: Vec<String>) -> Self {
        Self {
            base: base.into(),
            client_id: client_id.into(),
            scopes,
            http: Client::new(),
        }
    }

    pub async fn start(&self) -> Result<DeviceStart> {
        let url = format!("{}/oauth2/device", self.base);
        let scope_str = self.scopes.join(" ");
        let resp = self
            .http
            .post(&url)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scopes", scope_str.as_str()),
            ])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AppError::Auth(format!("device start {}", resp.status())));
        }
        Ok(resp.json().await?)
    }

    pub async fn poll(&self, device_code: &str) -> Result<StoredTokens> {
        let url = format!("{}/oauth2/token", self.base);
        loop {
            let resp = self
                .http
                .post(&url)
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await?;
            if resp.status().is_success() {
                return Ok(resp.json().await?);
            }
            let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
            let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
            if msg.contains("authorization_pending") || msg.contains("slow_down") {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            return Err(AppError::Auth(format!("poll failed: {body}")));
        }
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<StoredTokens> {
        let url = format!("{}/oauth2/token", self.base);
        let resp = self
            .http
            .post(&url)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AppError::Auth(format!("refresh {}", resp.status())));
        }
        Ok(resp.json().await?)
    }
}

const KEYRING_SERVICE: &str = "twitch-sankagata-manager";
const LEGACY_KEYRING_SERVICE: &str = "mumamuma-sankagata-manager";
const KEYRING_ACCOUNT: &str = "twitch";

fn keyring_entry(service: &str) -> std::result::Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(service, KEYRING_ACCOUNT)
}

pub fn store_tokens(tokens: &StoredTokens) -> Result<()> {
    let entry = keyring_entry(KEYRING_SERVICE)?;
    entry.set_password(&serde_json::to_string(tokens)?)?;
    Ok(())
}

pub fn load_tokens() -> Result<Option<StoredTokens>> {
    let entry = keyring_entry(KEYRING_SERVICE)?;
    match entry.get_password() {
        Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
        Err(keyring::Error::NoEntry) => load_legacy_tokens(),
        Err(e) => Err(AppError::Keyring(e)),
    }
}

pub fn clear_tokens() -> Result<()> {
    keyring_entry(KEYRING_SERVICE)?.delete_credential().ok();
    keyring_entry(LEGACY_KEYRING_SERVICE)?
        .delete_credential()
        .ok();
    Ok(())
}

fn load_legacy_tokens() -> Result<Option<StoredTokens>> {
    let entry = keyring_entry(LEGACY_KEYRING_SERVICE)?;
    match entry.get_password() {
        Ok(s) => {
            let tokens = serde_json::from_str(&s)?;
            store_tokens(&tokens)?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e)),
    }
}

/// Serialised refresh entry-point.
///
/// Twitch rotates `refresh_token` on every successful refresh, so two
/// concurrent callers using the same stored refresh_token race: the first
/// wins, the second hits "invalid refresh token" and the user appears
/// logged out. This guard funnels every refresh through one critical
/// section, and re-loads tokens after acquiring the lock so a caller who
/// waited can short-circuit on the freshly-stored tokens.
pub async fn refresh_under_guard(
    id_base: &str,
    client_id: &str,
    guard: &RefreshGuard,
) -> Result<StoredTokens> {
    let _g = guard.lock().await;
    let tokens = load_tokens()?.ok_or(AppError::NotAuthenticated)?;
    if tokens.is_fresh(60) {
        return Ok(tokens);
    }
    let flow = DeviceFlow::new(id_base, client_id, vec![]);
    let new_tokens = flow.refresh(&tokens.refresh_token).await?.stamp_now();
    store_tokens(&new_tokens)?;
    Ok(new_tokens)
}
