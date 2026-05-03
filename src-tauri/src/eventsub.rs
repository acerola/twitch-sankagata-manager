use crate::error::{AppError, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;

#[allow(dead_code)]
pub const PROD_URL: &str = "wss://eventsub.wss.twitch.tv/ws";
#[allow(dead_code)]
pub const MOCK_URL: &str = "ws://localhost:8081/ws";

#[derive(Debug, Clone, Deserialize)]
pub struct Redemption {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub reward: RewardRef,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RewardRef {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub enum EventSubMessage {
    Welcome {
        session_id: String,
        reconnect_url: Option<String>,
    },
    Notification {
        subscription_type: String,
        event: serde_json::Value,
    },
    Keepalive,
    Reconnect {
        new_url: String,
    },
    Revocation,
}

pub fn make_restart_channel() -> (
    tokio::sync::watch::Sender<()>,
    tokio::sync::watch::Receiver<()>,
) {
    tokio::sync::watch::channel(())
}

/// URL provider type - returns the current URL based on config
pub type UrlProvider = Arc<dyn Fn() -> String + Send + Sync>;

pub struct EventSubClient {
    url_provider: UrlProvider,
    tx: UnboundedSender<EventSubMessage>,
    restart_rx: Option<tokio::sync::watch::Receiver<()>>,
}

impl EventSubClient {
    pub fn new(url: impl Into<String>, tx: UnboundedSender<EventSubMessage>) -> Self {
        let url_str = url.into();
        Self {
            url_provider: Arc::new(move || url_str.clone()),
            tx,
            restart_rx: None,
        }
    }

    pub fn with_url_provider(
        url_provider: UrlProvider,
        tx: UnboundedSender<EventSubMessage>,
    ) -> Self {
        Self {
            url_provider,
            tx,
            restart_rx: None,
        }
    }

    pub fn with_restart(mut self, rx: tokio::sync::watch::Receiver<()>) -> Self {
        self.restart_rx = Some(rx);
        self
    }

    pub async fn run(mut self) -> Result<()> {
        let mut backoff_secs: u64 = 1;
        let mut reconnect_url: Option<String> = None;
        loop {
            let current_url = reconnect_url
                .clone()
                .unwrap_or_else(|| (self.url_provider)());
            tracing::info!("eventsub: connecting to {}", current_url);

            enum Outcome {
                Migrate(String),
                Finished,
                Restart,
                Err(AppError),
            }
            let outcome = if let Some(rx) = self.restart_rx.as_mut() {
                tokio::select! {
                    r = Self::once_inner(&current_url, &self.tx) => match r {
                        Ok(Some(u)) => Outcome::Migrate(u),
                        Ok(None) => Outcome::Finished,
                        Err(e) => Outcome::Err(e),
                    },
                    _ = rx.changed() => Outcome::Restart,
                }
            } else {
                match Self::once_inner(&current_url, &self.tx).await {
                    Ok(Some(u)) => Outcome::Migrate(u),
                    Ok(None) => Outcome::Finished,
                    Err(e) => Outcome::Err(e),
                }
            };
            match outcome {
                Outcome::Migrate(new_url) => {
                    tracing::info!("eventsub: migrating to {}", new_url);
                    reconnect_url = Some(new_url);
                    backoff_secs = 1;
                }
                Outcome::Finished => return Ok(()),
                Outcome::Restart => {
                    tracing::info!("eventsub: restart signal received, reconnecting...");
                    reconnect_url = None;
                    backoff_secs = 1;
                }
                Outcome::Err(_e) => {
                    tracing::warn!(
                        "eventsub: disconnected from {}, reconnecting in {}s",
                        current_url,
                        backoff_secs
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(60);
                }
            }
        }
    }

    async fn once_inner(
        url: &str,
        tx: &UnboundedSender<EventSubMessage>,
    ) -> Result<Option<String>> {
        let (mut ws, _) = connect_async(url)
            .await
            .map_err(|e| AppError::Other(e.to_string()))?;
        while let Some(msg) = ws.next().await {
            let msg = msg.map_err(|e| AppError::Other(e.to_string()))?;
            if let tokio_tungstenite::tungstenite::Message::Text(t) = msg {
                let v: serde_json::Value = serde_json::from_str(&t)?;
                let mtype = v["metadata"]["message_type"].as_str().unwrap_or("");
                match mtype {
                    "session_welcome" => {
                        let sid = v["payload"]["session"]["id"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let _ = tx.send(EventSubMessage::Welcome {
                            session_id: sid,
                            reconnect_url: None,
                        });
                    }
                    "session_keepalive" => {
                        let _ = tx.send(EventSubMessage::Keepalive);
                    }
                    "notification" => {
                        let stype = v["metadata"]["subscription_type"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let event = v["payload"]["event"].clone();
                        let _ = tx.send(EventSubMessage::Notification {
                            subscription_type: stype,
                            event,
                        });
                    }
                    "session_reconnect" => {
                        let new_url = v["payload"]["session"]["reconnect_url"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        if new_url.is_empty() {
                            return Err(AppError::Other(
                                "session_reconnect missing reconnect_url".into(),
                            ));
                        }
                        let _ = tx.send(EventSubMessage::Reconnect {
                            new_url: new_url.clone(),
                        });
                        ws.close(None).await.ok();
                        return Ok(Some(new_url));
                    }
                    "revocation" => {
                        let _ = tx.send(EventSubMessage::Revocation);
                    }
                    _ => {}
                }
            }
        }
        Err(AppError::Other("websocket closed".into()))
    }
}

pub async fn subscribe(
    helix_base: &str,
    client_id: &str,
    token: &str,
    session_id: &str,
    broadcaster_id: &str,
    event_type: &str,
) -> Result<()> {
    let http = reqwest::Client::new();
    let body = serde_json::json!({
        "type": event_type,
        "version": "1",
        "condition": { "broadcaster_user_id": broadcaster_id },
        "transport": { "method": "websocket", "session_id": session_id },
    });
    let resp = http
        .post(format!("{helix_base}/helix/eventsub/subscriptions"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Client-Id", client_id)
        .json(&body)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(AppError::Twitch(format!(
            "subscribe {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }
    Ok(())
}
