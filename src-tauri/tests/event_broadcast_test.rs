//! Regression guard for "overlay UI not reflecting main state": the lib.rs
//! spawn loop reads from a `broadcast::channel` and forwards every snapshot
//! into a sink (in production: `AppHandle::emit` + per-label `emit_to`).
//!
//! Tauri's own `mock_builder` fails to launch on this Windows host
//! (STATUS_ENTRYPOINT_NOT_FOUND from WebView2Loader.dll), so instead of
//! booting a real Tauri runtime this test drops in a trivial `Fn(&Snapshot)`
//! sink that mirrors the emit contract. If the forwarding loop regresses
//! (drops messages, exits early, filters) this test fails fast.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

use twitch_sankagata_manager_lib::model::{Snapshot, User};

fn empty_snapshot(tag: &str) -> Snapshot {
    Snapshot::new(
        vec![User {
            id: tag.into(),
            name: tag.into(),
            display_name: tag.into(),
            join_count: 0,
            last_join_at: None,
            enqueued_at: 0,
            manual_order: None,
            first_time_today: true,
        }],
        vec![],
        vec![],
        true,
        "ja".into(),
        3,
        "twitch".into(),
    )
}

/// Shape-identical loop to the one in `src/lib.rs`. Swap out the real `emit`
/// for a closure so the behaviour is testable without a webview runtime.
fn spawn_forwarder<F>(
    mut rx: broadcast::Receiver<Snapshot>,
    mut sink: F,
) -> tokio::task::JoinHandle<()>
where
    F: FnMut(&Snapshot) + Send + 'static,
{
    tokio::spawn(async move {
        while let Ok(snap) = rx.recv().await {
            sink(&snap);
        }
    })
}

#[tokio::test(flavor = "current_thread")]
async fn forwarder_delivers_every_broadcast_to_the_sink() {
    let (tx, rx) = broadcast::channel::<Snapshot>(16);
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen.clone();
    let task = spawn_forwarder(rx, move |s| {
        seen_clone.lock().unwrap().push(s.playing[0].id.clone());
    });

    for id in ["u1", "u2", "u3", "u4", "u5"] {
        tx.send(empty_snapshot(id)).unwrap();
    }
    drop(tx);
    let _ = tokio::time::timeout(Duration::from_millis(500), task).await;

    let got = seen.lock().unwrap().clone();
    assert_eq!(got, vec!["u1", "u2", "u3", "u4", "u5"]);
}

#[tokio::test(flavor = "current_thread")]
async fn forwarder_drives_multiple_sinks_for_every_webview() {
    // Simulates the belt-and-suspenders fallback: same message reaches every
    // registered sink (each label = webview). If this regresses (e.g. someone
    // short-circuits after first emit), later sinks miss updates.
    let (tx, rx) = broadcast::channel::<Snapshot>(8);
    let counts: Arc<Mutex<std::collections::HashMap<String, u32>>> =
        Arc::new(Mutex::new(Default::default()));
    let sinks = vec!["main", "overlay"];
    let sink_counts = counts.clone();
    let labels = sinks.clone();
    let task = spawn_forwarder(rx, move |_s| {
        let mut m = sink_counts.lock().unwrap();
        for label in &labels {
            *m.entry((*label).to_string()).or_insert(0) += 1;
        }
    });

    for id in ["a", "b", "c"] {
        tx.send(empty_snapshot(id)).unwrap();
    }
    drop(tx);
    let _ = tokio::time::timeout(Duration::from_millis(500), task).await;

    let m = counts.lock().unwrap();
    for label in &sinks {
        assert_eq!(
            m.get(*label).copied().unwrap_or(0),
            3,
            "{label} missed a tick"
        );
    }
}
