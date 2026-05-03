use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reward {
    pub id: String,
    pub title: String,
    pub cost: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Clone)]
pub struct HelixClient {
    base: String,
    client_id: String,
    token: String,
    http: Client,
}

impl HelixClient {
    pub fn new(
        base: impl Into<String>,
        client_id: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            base: base.into(),
            client_id: client_id.into(),
            token: token.into(),
            http: Client::new(),
        }
    }

    pub async fn get_self(&self) -> Result<TwitchUser> {
        #[derive(Deserialize)]
        struct Resp {
            data: Vec<TwitchUser>,
        }
        let url = format!("{}/helix/users", self.base);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-Id", &self.client_id)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Twitch(format!("get_self {status}: {body}")));
        }
        resp.json::<Resp>()
            .await?
            .data
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Twitch("get_self: empty data".into()))
    }

    pub async fn list_rewards(&self, broadcaster_id: &str) -> Result<Vec<Reward>> {
        #[derive(Deserialize)]
        struct Resp {
            data: Vec<Reward>,
        }
        let url = format!("{}/helix/channel_points/custom_rewards", self.base);
        let resp = self
            .http
            .get(&url)
            .query(&[("broadcaster_id", broadcaster_id)])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-Id", &self.client_id)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Twitch(format!("list_rewards {status}: {body}")));
        }
        Ok(resp.json::<Resp>().await?.data)
    }

    pub async fn refund_redemption(
        &self,
        broadcaster_id: &str,
        reward_id: &str,
        redemption_id: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/helix/channel_points/custom_rewards/redemptions",
            self.base
        );
        let resp = self
            .http
            .patch(&url)
            .query(&[
                ("broadcaster_id", broadcaster_id),
                ("reward_id", reward_id),
                ("id", redemption_id),
            ])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-Id", &self.client_id)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"CANCELED"}"#)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AppError::Twitch(format!(
                "refund {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        Ok(())
    }
}
