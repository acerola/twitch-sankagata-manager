use crate::auth::{load_tokens, refresh_under_guard, RefreshGuard};
use crate::eventsub::{subscribe, EventSubMessage, Redemption};
use crate::filter::{should_accept, ParsedRedemption};
use crate::model::Snapshot;
use crate::state::{now_ms, AppState};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[derive(Clone)]
pub struct HelixCtx {
    pub helix_base: String,
    pub id_base: String,
    pub client_id: String,
}

async fn fetch_broadcaster_id(helix_base: &str, client_id: &str, token: &str) -> Option<String> {
    let http = reqwest::Client::new();
    let resp = http
        .get(format!("{helix_base}/helix/users"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Client-Id", client_id)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    body["data"][0]["id"].as_str().map(String::from)
}

pub async fn run_pipeline(
    state: Arc<AppState>,
    mut rx: mpsc::UnboundedReceiver<EventSubMessage>,
    tx: broadcast::Sender<Snapshot>,
    helix: Option<HelixCtx>,
    refresh_guard: RefreshGuard,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            EventSubMessage::Welcome { session_id, .. } => {
                tracing::info!("EventSub session_id: {session_id}");
                state.set_session_id(session_id.clone());
                let Some(ref ctx) = helix else { continue };
                let mut tokens = match load_tokens() {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        tracing::info!("session_welcome received, no tokens — skipping subscribe");
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("load_tokens failed: {e} — skipping subscribe");
                        continue;
                    }
                };
                let broadcaster_id = match fetch_broadcaster_id(
                    &ctx.helix_base,
                    &ctx.client_id,
                    &tokens.access_token,
                )
                .await
                {
                    Some(id) => id,
                    None => {
                        tracing::info!("fetch_broadcaster_id failed, attempting token refresh...");
                        match refresh_under_guard(&ctx.id_base, &ctx.client_id, &refresh_guard)
                            .await
                        {
                            Ok(new_tokens) => {
                                tokens = new_tokens;
                                match fetch_broadcaster_id(
                                    &ctx.helix_base,
                                    &ctx.client_id,
                                    &tokens.access_token,
                                )
                                .await
                                {
                                    Some(id) => id,
                                    None => {
                                        tracing::warn!("still could not fetch broadcaster_id after refresh — skipping subscribe");
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("token refresh failed: {e} — skipping subscribe");
                                continue;
                            }
                        }
                    }
                };
                for event_type in &[
                    "channel.channel_points_custom_reward_redemption.add",
                    "channel.channel_points_custom_reward_redemption.update",
                ] {
                    if let Err(e) = subscribe(
                        &ctx.helix_base,
                        &ctx.client_id,
                        &tokens.access_token,
                        &session_id,
                        &broadcaster_id,
                        event_type,
                    )
                    .await
                    {
                        tracing::warn!("subscribe {event_type} failed: {e}");
                    } else {
                        tracing::info!("subscribed to {event_type}");
                    }
                }
            }
            EventSubMessage::Notification {
                subscription_type,
                event,
            } => {
                if subscription_type == "channel.channel_points_custom_reward_redemption.add" {
                    if let Ok(r) = serde_json::from_value::<Redemption>(event) {
                        let cfg = state.config();
                        if !should_accept(&r, &cfg) {
                            continue;
                        }
                        let parsed = ParsedRedemption::from(&r);
                        let _ = state.add_redemption(parsed.user, now_ms());
                        let _ = tx.send(state.snapshot());
                    }
                } else if subscription_type
                    == "channel.channel_points_custom_reward_redemption.update"
                {
                    if let Ok(r) = serde_json::from_value::<Redemption>(event) {
                        if r.status == "CANCELED" {
                            let _ = state.refund_user(&r.user_id);
                            let _ = tx.send(state.snapshot());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
