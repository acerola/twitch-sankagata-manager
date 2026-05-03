use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::{broadcast, mpsc};
use twitch_sankagata_manager_lib::auth::make_refresh_guard;
use twitch_sankagata_manager_lib::eventsub::EventSubMessage;
use twitch_sankagata_manager_lib::model::Snapshot;
use twitch_sankagata_manager_lib::pipeline::run_pipeline;
use twitch_sankagata_manager_lib::state::AppState;
use twitch_sankagata_manager_lib::store::Store;

#[tokio::test]
async fn pipeline_adds_user_on_valid_redemption() {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    let state = Arc::new(AppState::new(store));
    let (tx_evt, rx_evt) = mpsc::unbounded_channel();
    let (tx_snap, _) = broadcast::channel::<Snapshot>(16);

    let state_clone = state.clone();
    let guard = make_refresh_guard();
    tokio::spawn(async move {
        run_pipeline(state_clone, rx_evt, tx_snap, None, guard).await;
    });

    // default keyword = 参加券, no reward_id → keyword fallback
    tx_evt
        .send(EventSubMessage::Notification {
            subscription_type: "channel.channel_points_custom_reward_redemption.add".into(),
            event: json!({
                "id": "rd1", "user_id": "u1", "user_name": "Alice", "user_login": "alice",
                "reward": { "id": "r1", "title": "参加券を使う" },
                "status": "UNFULFILLED"
            }),
        })
        .unwrap();

    // give pipeline a tick
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let snap = state.snapshot();
    assert_eq!(snap.playing.len() + snap.waiting.len(), 1);
    let u = snap
        .playing
        .iter()
        .chain(snap.waiting.iter())
        .next()
        .unwrap();
    assert_eq!(u.display_name, "Alice");
}

#[tokio::test]
async fn pipeline_refunds_user_on_canceled_update() {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    let state = Arc::new(AppState::new(store));
    let (tx_evt, rx_evt) = mpsc::unbounded_channel();
    let (tx_snap, _) = broadcast::channel::<Snapshot>(16);

    let state_clone = state.clone();
    let guard = make_refresh_guard();
    tokio::spawn(async move {
        run_pipeline(state_clone, rx_evt, tx_snap, None, guard).await;
    });

    tx_evt
        .send(EventSubMessage::Notification {
            subscription_type: "channel.channel_points_custom_reward_redemption.add".into(),
            event: json!({
                "id": "rd1", "user_id": "u1", "user_name": "Alice", "user_login": "alice",
                "reward": { "id": "r1", "title": "参加券を使う" },
                "status": "UNFULFILLED"
            }),
        })
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    tx_evt
        .send(EventSubMessage::Notification {
            subscription_type: "channel.channel_points_custom_reward_redemption.update".into(),
            event: json!({
                "id": "rd1", "user_id": "u1", "user_name": "Alice", "user_login": "alice",
                "reward": { "id": "r1", "title": "参加券を使う" },
                "status": "CANCELED"
            }),
        })
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let snap = state.snapshot();
    assert_eq!(snap.playing.len() + snap.waiting.len(), 0);
}
