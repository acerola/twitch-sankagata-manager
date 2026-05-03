use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tempfile::tempdir;
use tower::ServiceExt;
use twitch_sankagata_manager_lib::model::Snapshot;
use twitch_sankagata_manager_lib::server::build_router;
use twitch_sankagata_manager_lib::state::AppState;
use twitch_sankagata_manager_lib::store::Store;

fn test_state() -> Arc<AppState> {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    Arc::new(AppState::new(store))
}

#[tokio::test]
async fn serves_overlay_html() {
    let router = build_router(
        test_state(),
        tokio::sync::broadcast::channel::<Snapshot>(16).0,
    );
    let resp = router
        .oneshot(
            Request::builder()
                .uri("/overlay")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1_000_000)
        .await
        .unwrap();
    assert!(String::from_utf8_lossy(&body).contains("<html"));
}

#[tokio::test]
async fn serves_overlay_css_and_js() {
    let router = build_router(
        test_state(),
        tokio::sync::broadcast::channel::<Snapshot>(16).0,
    );
    let css = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/overlay.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(css.status(), StatusCode::OK);
    let js = router
        .oneshot(
            Request::builder()
                .uri("/overlay.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(js.status(), StatusCode::OK);
}
