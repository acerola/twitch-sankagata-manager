use crate::config::Config;
use crate::error::AppError;
use crate::helix::HelixClient;
use crate::model::{Snapshot, Zone};
use crate::state::AppState;
use serde::Serialize;
#[cfg(target_os = "windows")]
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use tokio::sync::broadcast;

pub type SnapTx<'a> = tauri::State<'a, broadcast::Sender<Snapshot>>;

fn broadcast_snapshot(tx: &broadcast::Sender<Snapshot>, snap: Snapshot) {
    let _ = tx.send(snap);
}

#[derive(Serialize)]
pub struct IpcError {
    message: String,
}

impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

pub type Ctx<'a> = State<'a, Arc<AppState>>;

#[tauri::command]
pub fn get_snapshot(state: Ctx<'_>) -> Snapshot {
    state.snapshot()
}

#[tauri::command]
pub fn get_config(state: Ctx<'_>) -> Config {
    state.config()
}

#[tauri::command]
pub fn get_session_id(state: Ctx<'_>) -> Option<String> {
    state.get_session_id()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallMode {
    pub kind: &'static str,
    pub portable: bool,
    pub detail: String,
}

#[tauri::command]
pub fn get_install_mode() -> InstallMode {
    detect_install_mode()
}

#[cfg(target_os = "windows")]
fn detect_install_mode() -> InstallMode {
    if std::env::var("SANKAGATA_FORCE_PORTABLE").ok().as_deref() == Some("1") {
        return InstallMode {
            kind: "portable",
            portable: true,
            detail: "forced by SANKAGATA_FORCE_PORTABLE".into(),
        };
    }

    let Ok(exe) = std::env::current_exe() else {
        return InstallMode {
            kind: "unknown",
            portable: false,
            detail: "could not resolve current executable".into(),
        };
    };
    let Some(dir) = exe.parent() else {
        return InstallMode {
            kind: "unknown",
            portable: false,
            detail: "current executable has no parent directory".into(),
        };
    };

    if dir.join("portable.txt").exists() || dir.join(".portable").exists() {
        return InstallMode {
            kind: "portable",
            portable: true,
            detail: format!("portable marker found in {}", dir.display()),
        };
    }

    if has_windows_uninstaller(dir) {
        return InstallMode {
            kind: "installed",
            portable: false,
            detail: format!("installer marker found in {}", dir.display()),
        };
    }

    InstallMode {
        kind: "portable",
        portable: true,
        detail: format!("no installer marker found in {}", dir.display()),
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_install_mode() -> InstallMode {
    InstallMode {
        kind: "installed",
        portable: false,
        detail: "non-Windows updater package".into(),
    }
}

#[cfg(target_os = "windows")]
fn has_windows_uninstaller(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .any(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return false;
            }
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            name.ends_with(".exe") && name.contains("uninstall")
        })
}

#[tauri::command]
pub fn set_config(
    state: Ctx<'_>,
    tx: SnapTx<'_>,
    cfg: Config,
    restart_tx: tauri::State<'_, tokio::sync::watch::Sender<()>>,
) -> Result<Snapshot, IpcError> {
    let old_mock = state.config().mock_mode;
    state.set_config(cfg)?;
    let new_mock = state.config().mock_mode;
    if old_mock != new_mock {
        let _ = restart_tx.send(());
        tracing::info!(
            "mock_mode toggled {} -> {}, sent restart signal",
            old_mock,
            new_mock
        );
    }
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn set_enabled(state: Ctx<'_>, tx: SnapTx<'_>, enabled: bool) -> Result<Snapshot, IpcError> {
    let mut cfg = state.config();
    cfg.enabled = enabled;
    state.set_config(cfg)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn trash_user(state: Ctx<'_>, tx: SnapTx<'_>, user_id: String) -> Result<Snapshot, IpcError> {
    state.trash_user(&user_id)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn restore_user(state: Ctx<'_>, tx: SnapTx<'_>, user_id: String) -> Result<Snapshot, IpcError> {
    state.restore_user(&user_id)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn clear_trash(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    state.clear_trash()?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn clear_playing(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    state.clear_playing()?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn clear_playing_user(
    state: Ctx<'_>,
    tx: SnapTx<'_>,
    user_id: String,
) -> Result<Snapshot, IpcError> {
    state.clear_playing_user(&user_id)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn move_user(
    state: Ctx<'_>,
    tx: SnapTx<'_>,
    user_id: String,
    zone: Zone,
    index: usize,
) -> Result<Snapshot, IpcError> {
    state.move_user(&user_id, zone, index)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub fn reset_counts(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    state.reset_counts()?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[tauri::command]
pub async fn list_rewards(
    cfg: tauri::State<'_, AuthConfig>,
) -> Result<Vec<crate::helix::Reward>, IpcError> {
    let tokens = load_tokens()
        .map_err(IpcError::from)?
        .ok_or_else(|| IpcError {
            message: "not authenticated".into(),
        })?;
    let client = HelixClient::new(&cfg.helix_base, &cfg.client_id, &tokens.access_token);
    let me = client.get_self().await.map_err(IpcError::from)?;
    // TODO: handle token refresh here too if 401
    client.list_rewards(&me.id).await.map_err(Into::into)
}

use crate::auth::{
    clear_tokens, load_tokens, refresh_under_guard, store_tokens, DeviceFlow, RefreshGuard,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub authenticated: bool,
    pub login_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStartInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
}

#[tauri::command]
pub async fn get_auth_status(
    cfg: tauri::State<'_, AuthConfig>,
    guard: tauri::State<'_, RefreshGuard>,
) -> Result<AuthStatus, IpcError> {
    let Some(tokens) = load_tokens().map_err(IpcError::from)? else {
        return Ok(AuthStatus {
            authenticated: false,
            login_name: None,
        });
    };

    let client = HelixClient::new(&cfg.helix_base, &cfg.client_id, &tokens.access_token);
    match client.get_self().await {
        Ok(u) => Ok(AuthStatus {
            authenticated: true,
            login_name: Some(u.display_name),
        }),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("401") {
                tracing::info!("access token expired (401), attempting refresh...");
                match refresh_under_guard(&cfg.id_base, &cfg.client_id, &guard).await {
                    Ok(new_tokens) => {
                        tracing::info!("token refresh successful");
                        let client = HelixClient::new(
                            &cfg.helix_base,
                            &cfg.client_id,
                            &new_tokens.access_token,
                        );
                        match client.get_self().await {
                            Ok(u) => Ok(AuthStatus {
                                authenticated: true,
                                login_name: Some(u.display_name),
                            }),
                            Err(e) => {
                                tracing::warn!("refreshed token validation failed: {e}");
                                Ok(AuthStatus {
                                    authenticated: false,
                                    login_name: None,
                                })
                            }
                        }
                    }
                    Err(refresh_err) => {
                        tracing::warn!("token refresh failed: {refresh_err}");
                        Ok(AuthStatus {
                            authenticated: false,
                            login_name: None,
                        })
                    }
                }
            } else {
                tracing::warn!("get_self failed with non-auth error: {e}");
                Ok(AuthStatus {
                    authenticated: false,
                    login_name: None,
                })
            }
        }
    }
}

#[tauri::command]
pub async fn start_auth(
    base: tauri::State<'_, AuthConfig>,
    restart_tx: tauri::State<'_, tokio::sync::watch::Sender<()>>,
) -> Result<DeviceStartInfo, IpcError> {
    let flow = DeviceFlow::new(
        &base.id_base,
        &base.client_id,
        vec![
            "channel:read:redemptions".into(),
            "channel:manage:redemptions".into(),
        ],
    );
    let start = flow.start().await.map_err(IpcError::from)?;
    // kick off polling in background
    let client_id = base.client_id.clone();
    let id_base = base.id_base.clone();
    let device_code = start.device_code.clone();
    let restart_tx_clone = (*restart_tx).clone();
    tokio::spawn(async move {
        let flow = DeviceFlow::new(&id_base, &client_id, vec![]);
        match flow.poll(&device_code).await {
            Ok(tokens) => {
                let tokens = tokens.stamp_now();
                if let Err(e) = store_tokens(&tokens) {
                    tracing::error!("store_tokens failed: {e}");
                    return;
                }
                // signal eventsub to reconnect so next Welcome triggers subscribe
                let _ = restart_tx_clone.send(());
                tracing::info!("auth success, sent restart signal to eventsub");
            }
            Err(e) => tracing::error!("device poll failed: {e}"),
        }
    });
    Ok(DeviceStartInfo {
        device_code: start.device_code,
        user_code: start.user_code,
        verification_uri: start.verification_uri,
    })
}

#[tauri::command]
pub async fn logout() -> Result<(), IpcError> {
    clear_tokens().map_err(IpcError::from)
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_seed_users(state: Ctx<'_>, tx: SnapTx<'_>, count: u32) -> Result<Snapshot, IpcError> {
    use crate::config::Language;
    use crate::model::User;
    use crate::state::now_ms;
    let base = now_ms();

    // Pick sample pool matching the streamer's configured language so seeded
    // rows look authentic in the UI they're actually staring at.
    let ja: &[&str] = &[
        "さくら",
        "ユウキ",
        "みなと",
        "れん",
        "ひかり",
        "はるか",
        "ちひろ",
        "長靴の猫",
        "まこと",
        "たろう",
        "はなこ",
        "ゲーム好き",
        "夜桜",
        "しおり",
        "あおい",
        "そら",
        "ダイスケ",
        "みお",
        "あきら",
        "カズ",
    ];
    let ko: &[&str] = &[
        "김민준",
        "이서연",
        "박하늘",
        "최지훈",
        "정유진",
        "강도윤",
        "윤서아",
        "조민서",
        "한예은",
        "임시우",
        "서지안",
        "배수빈",
        "송다은",
        "장재원",
        "오현우",
        "구하율",
        "신아린",
        "문지호",
        "황태영",
        "권소율",
    ];
    let en: &[&str] = &[
        "Alice", "Bob", "Charlie", "Dave", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack", "Kate",
        "Liam", "Mia", "Noah", "Olivia", "Pete", "Quinn", "Ruby", "Sam", "Tina",
    ];
    let pool: &[&str] = match state.config().language {
        Language::Ja => ja,
        Language::Ko => ko,
        Language::En => en,
    };

    use rand::seq::SliceRandom;
    use rand::Rng;
    // Shuffle pool indices so successive debug clicks pick different names
    // and same-batch rows never visually duplicate.
    let mut rng = rand::thread_rng();
    let mut shuffled: Vec<usize> = (0..pool.len()).collect();
    shuffled.shuffle(&mut rng);
    // Per-batch suffix keeps cross-batch display names distinct even when the
    // pool wraps (count > pool.len()).
    let batch_tag: u16 = rng.gen_range(100..1000);

    let repeat_cutoff = (count as usize) / 2;
    for i in 0..count as usize {
        let name = pool[shuffled[i % pool.len()]];
        let is_repeat = i < repeat_cutoff;
        let u = User {
            id: format!("dbg-{}-{i}", base),
            name: format!("seed{i}"),
            display_name: format!("{name}{batch_tag}{i:02}"),
            join_count: if is_repeat { 2 } else { 0 },
            last_join_at: if is_repeat {
                Some(base - 60 * 60 * 1000)
            } else {
                None
            },
            enqueued_at: base + i as i64,
            manual_order: None,
            first_time_today: !is_repeat,
        };
        state.add_redemption(u, base + i as i64)?;
    }
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn debug_seed_users(_count: u32) -> Result<(), IpcError> {
    Err(IpcError {
        message: "debug_seed_users disabled in release builds".into(),
    })
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_seed_long_names(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    use crate::model::User;
    use crate::state::now_ms;
    use rand::Rng;
    let base = now_ms();
    let mut rng = rand::thread_rng();
    let batch_tag: u16 = rng.gen_range(100..1000);

    // Variety of pathological lengths so ellipsis can be eyeballed at every break.
    let samples = [
        "ShortName",
        "MidLengthUsername01",
        "this_is_a_very_long_user_name_2026",
        "the_user_who_just_kept_typing_more_letters_forever",
        "あいうえおかきくけこさしすせそたちつてとなにぬねの",
        "한국어로_엄청나게_긴_사용자_이름_테스트",
        "🎮GamerNeko_with_many_emojis_🌸✨🐈‍⬛_streamer_2026",
        "ALL_CAPS_USERNAME_SCREAMING_INTO_THE_VOID",
        "x",
        "kawaii.streamer.tonight.0928.evening.session",
    ];

    let repeat_cutoff = samples.len() / 2;
    for (i, sample) in samples.iter().enumerate() {
        let is_repeat = i < repeat_cutoff;
        let u = User {
            id: format!("dbg-long-{}-{i}", base),
            name: format!("longseed{i}"),
            display_name: format!("{sample}#{batch_tag}"),
            join_count: if is_repeat { 2 } else { 0 },
            last_join_at: if is_repeat {
                Some(base - 60 * 60 * 1000)
            } else {
                None
            },
            enqueued_at: base + i as i64,
            manual_order: None,
            first_time_today: !is_repeat,
        };
        state.add_redemption(u, base + i as i64)?;
    }
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn debug_seed_long_names() -> Result<(), IpcError> {
    Err(IpcError {
        message: "debug_seed_long_names disabled in release builds".into(),
    })
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_clear_queues(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    state.debug_clear_all()?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn debug_clear_queues() -> Result<(), IpcError> {
    Err(IpcError {
        message: "debug_clear_queues disabled in release builds".into(),
    })
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_refund_first(state: Ctx<'_>, tx: SnapTx<'_>) -> Result<Snapshot, IpcError> {
    state.debug_refund_first_playing()?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn debug_refund_first() -> Result<(), IpcError> {
    Err(IpcError {
        message: "debug_refund_first disabled in release builds".into(),
    })
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_trigger_mock_redemption(
    state: Ctx<'_>,
    tx: SnapTx<'_>,
    _title: String,
) -> Result<Snapshot, IpcError> {
    use crate::model::User;
    use crate::state::now_ms;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let base = now_ms();
    let seq: u32 = rng.gen_range(0..10_000);
    let user = User {
        id: format!("mock-{base}-{seq}"),
        name: format!("mockuser{seq}"),
        display_name: format!("MockUser#{seq}"),
        join_count: 0,
        last_join_at: None,
        enqueued_at: base,
        manual_order: None,
        first_time_today: true,
    };
    state.add_redemption(user, base)?;
    let snap = state.snapshot();
    broadcast_snapshot(&tx, snap.clone());
    Ok(snap)
}

#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn debug_trigger_mock_redemption(_title: String) -> Result<(), IpcError> {
    Err(IpcError {
        message: "debug_trigger_mock_redemption disabled in release builds".into(),
    })
}

pub struct AuthConfig {
    pub id_base: String,
    pub helix_base: String,
    pub client_id: String,
}
