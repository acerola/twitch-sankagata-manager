use crate::model::Snapshot;
use crate::overlay_assets::*;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppCtx {
    pub state: Arc<AppState>,
    pub tx: broadcast::Sender<Snapshot>,
}

pub fn build_router(state: Arc<AppState>, tx: broadcast::Sender<Snapshot>) -> Router {
    let ctx = AppCtx { state, tx };
    Router::new()
        .route("/overlay", get(overlay_html))
        .route("/overlay.css", get(overlay_css))
        .route("/overlay.js", get(overlay_js))
        .route("/ws", get(ws_handler))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(ctx)
}

// `no-store` defeats OBS Browser Source's CEF disk cache, which otherwise pins
// the first-served CSS/JS forever — old colors/gradients survive across rebuilds.
const NO_CACHE: &str = "no-store, no-cache, must-revalidate, max-age=0";

async fn overlay_html() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, NO_CACHE),
            (header::PRAGMA, "no-cache"),
        ],
        OVERLAY_HTML,
    )
        .into_response()
}
async fn overlay_css() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, NO_CACHE),
            (header::PRAGMA, "no-cache"),
        ],
        format!("{}\n{}", THEMES_CSS, OVERLAY_CSS),
    )
        .into_response()
}
async fn overlay_js() -> Response {
    (
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, NO_CACHE),
            (header::PRAGMA, "no-cache"),
        ],
        OVERLAY_JS,
    )
        .into_response()
}

async fn ws_handler(State(ctx): State<AppCtx>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |sock| handle_socket(sock, ctx))
}

async fn handle_socket(mut sock: WebSocket, ctx: AppCtx) {
    let snap = ctx.state.snapshot();
    let _ = sock
        .send(Message::Text(
            serde_json::to_string(&snap).unwrap_or_default(),
        ))
        .await;
    let mut rx = ctx.tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(s) => {
                        if sock.send(Message::Text(serde_json::to_string(&s).unwrap_or_default())).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => { continue; }
                    Err(_) => { break; }
                }
            }
            incoming = sock.recv() => {
                match incoming {
                    Some(Ok(_)) => continue,
                    _ => break,
                }
            }
        }
    }
}

pub async fn bind_with_fallback(
    start_port: u16,
) -> std::io::Result<(tokio::net::TcpListener, u16)> {
    for p in start_port..=start_port + 10 {
        match tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
            Ok(l) => return Ok((l, p)),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "no free port in range",
    ))
}
