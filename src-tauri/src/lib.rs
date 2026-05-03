pub mod auth;
pub mod config;
pub mod error;
pub mod eventsub;
pub mod filter;
pub mod helix;
pub mod ipc;
pub mod model;
pub mod overlay_assets;
pub mod pipeline;
pub mod priority;
pub mod server;
pub mod state;
pub mod store;

use std::path::Path;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::{broadcast, mpsc};

pub const TWITCH_CLIENT_ID: &str = env!("TWITCH_CLIENT_ID");
pub const TWITCH_API_BASE: &str = env!("TWITCH_API_BASE");
pub const TWITCH_ID_BASE: &str = env!("TWITCH_ID_BASE");
pub const TWITCH_EVENTSUB_URL: &str = env!("TWITCH_EVENTSUB_URL");
pub const TWITCH_EVENTSUB_MOCK_URL: &str = "ws://127.0.0.1:8081/ws";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = handle.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&data_dir).ok();
            migrate_legacy_data_dir(&data_dir);
            let db_path = data_dir.join("state.db");
            let store = Arc::new(store::Store::open(&db_path).expect("open db"));
            let app_state = Arc::new(state::AppState::new(store));
            app.manage(app_state.clone());

            let (tx_evt, rx_evt) = mpsc::unbounded_channel();
            let (tx_snap, _rx0) = broadcast::channel::<model::Snapshot>(64);
            app.manage(tx_snap.clone());
            app.manage(ipc::AuthConfig {
                id_base: TWITCH_ID_BASE.into(),
                helix_base: TWITCH_API_BASE.into(),
                client_id: TWITCH_CLIENT_ID.into(),
            });
            let refresh_guard = auth::make_refresh_guard();
            app.manage(refresh_guard.clone());

            // forward snapshot broadcasts to every webview. Using emit_to per label
            // (main/overlay) guarantees delivery even if a global emit misses
            // a secondary window during its async listen() registration.
            let handle_for_emit = handle.clone();
            let mut snap_rx = tx_snap.subscribe();
            tauri::async_runtime::spawn(async move {
                while let Ok(snap) = snap_rx.recv().await {
                    // 1) global — catches any window that registered a default listener
                    if let Err(e) = handle_for_emit.emit("state-changed", &snap) {
                        tracing::error!("emit state-changed (global) failed: {e}");
                    }
                    // 2) explicit per-window — bullet-proof fallback
                    for (label, _w) in handle_for_emit.webview_windows().iter() {
                        if let Err(e) =
                            handle_for_emit.emit_to(label.as_str(), "state-changed", &snap)
                        {
                            tracing::warn!("emit_to {label} failed: {e}");
                        }
                    }
                    tracing::debug!(
                        "state-changed emitted: playing={} waiting={} trash={}",
                        snap.playing.len(),
                        snap.waiting.len(),
                        snap.trash.len()
                    );
                }
            });

            // http server — bind at the port the user saved, not the hardcoded default.
            // If fallback bumps the port, write the actual port back into config so the
            // header's Copy-OBS-URL button always reflects the real bound port.
            let state_for_server = app_state.clone();
            let tx_snap_for_server = tx_snap.clone();
            let requested_port = app_state.config().port;
            // Channel fired from RunEvent::ExitRequested below so axum drops the
            // listener immediately on app quit. Without this the OS may hold the
            // socket in TIME_WAIT and the next launch falls back to port+1,
            // leaving OBS pointed at a phantom URL.
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            app.manage(ServerShutdown(std::sync::Mutex::new(Some(shutdown_tx))));
            tauri::async_runtime::spawn(async move {
                let router = server::build_router(state_for_server.clone(), tx_snap_for_server);
                match server::bind_with_fallback(requested_port).await {
                    Ok((listener, actual)) => {
                        tracing::info!("overlay at http://localhost:{actual}/overlay");
                        if actual != requested_port {
                            tracing::warn!(
                                "requested port {requested_port} was busy — bound {actual}. \
                                 if OBS shows stale UI, kill any leftover process holding {requested_port}."
                            );
                            let mut cfg = state_for_server.config();
                            cfg.port = actual;
                            let _ = state_for_server.set_config(cfg);
                        }
                        let _ = axum::serve(listener, router)
                            .with_graceful_shutdown(async move {
                                let _ = shutdown_rx.await;
                                tracing::info!("axum graceful shutdown");
                            })
                            .await;
                    }
                    Err(e) => tracing::error!("server failed: {e}"),
                }
            });

            let helix_ctx = pipeline::HelixCtx {
                helix_base: TWITCH_API_BASE.into(),
                id_base: TWITCH_ID_BASE.into(),
                client_id: TWITCH_CLIENT_ID.into(),
            };

            // pipeline — always receives Some(HelixCtx); looks up tokens/broadcaster_id dynamically
            let state_for_pipeline = app_state.clone();
            let tx_snap_for_pipeline = tx_snap.clone();
            let helix_for_pipeline = helix_ctx.clone();
            let refresh_guard_for_pipeline = refresh_guard.clone();
            tauri::async_runtime::spawn(async move {
                pipeline::run_pipeline(
                    state_for_pipeline,
                    rx_evt,
                    tx_snap_for_pipeline,
                    Some(helix_for_pipeline),
                    refresh_guard_for_pipeline,
                )
                .await;
            });

            // restart channel so auth success can force eventsub to reconnect
            let (restart_tx, restart_rx) = eventsub::make_restart_channel();
            app.manage(restart_tx);

            // eventsub — always spawn; pipeline handles unauthenticated gracefully
            // URL is dynamic based on mock_mode in config
            let tx_evt_clone = tx_evt.clone();
            let state_for_eventsub = app_state.clone();
            let url_provider = std::sync::Arc::new(move || {
                if state_for_eventsub.config().mock_mode {
                    TWITCH_EVENTSUB_MOCK_URL.to_string()
                } else {
                    TWITCH_EVENTSUB_URL.to_string()
                }
            });
            let client = eventsub::EventSubClient::with_url_provider(url_provider, tx_evt_clone)
                .with_restart(restart_rx);
            tauri::async_runtime::spawn(async move {
                let _ = client.run().await;
            });

            // keep sender alive so pipeline receiver doesn't close
            let _tx_evt_keep = tx_evt;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::get_snapshot,
            ipc::get_config,
            ipc::get_session_id,
            ipc::get_install_mode,
            ipc::set_config,
            ipc::set_enabled,
            ipc::trash_user,
            ipc::restore_user,
            ipc::clear_trash,
            ipc::clear_playing,
            ipc::clear_playing_user,
            ipc::move_user,
            ipc::reset_counts,
            ipc::list_rewards,
            ipc::get_auth_status,
            ipc::start_auth,
            ipc::logout,
            ipc::debug_seed_users,
            ipc::debug_seed_long_names,
            ipc::debug_clear_queues,
            ipc::debug_refund_first,
            ipc::debug_trigger_mock_redemption,
        ])
        .build(tauri::generate_context!())
        .expect("error while building application")
        .run(|app, event| match event {
            // Closing main = closing the app. Without this, killing main on
            // Windows can leave the overlay window orphaned + the process alive.
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { .. },
                ..
            } if label == "main" => {
                app.exit(0);
            }
            tauri::RunEvent::ExitRequested { .. } => {
                if let Some(s) = app.try_state::<ServerShutdown>() {
                    if let Some(tx) = s.0.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                }
            }
            _ => {}
        });
}

fn migrate_legacy_data_dir(data_dir: &Path) {
    if data_dir.join("state.db").exists() {
        return;
    }

    let Some(parent) = data_dir.parent() else {
        return;
    };

    for legacy_dir_name in [
        "com.mumamuma.sankagata.manager",
        "Mumamuma Sankagata Manager",
        "mumamuma-sankagata-manager",
    ] {
        let legacy_db = parent.join(legacy_dir_name).join("state.db");
        if !legacy_db.exists() {
            continue;
        }

        match std::fs::copy(&legacy_db, data_dir.join("state.db")) {
            Ok(_) => tracing::info!("migrated state database from {legacy_dir_name}"),
            Err(e) => {
                tracing::warn!("failed to migrate state database from {legacy_dir_name}: {e}")
            }
        }
        break;
    }
}

struct ServerShutdown(std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>);
