# Twitch Sankagata Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2.x desktop app that detects Twitch channel-point redemptions, manages a playing/waiting/trash queue with manual drag-drop, and serves a transparent OBS overlay at `http://localhost:24816/overlay`.

**Architecture:** Single Rust backend process (tokio + axum + rusqlite + keyring) owns all state and Twitch connections. Two Tauri webview windows (main = Playing|Waiting, separate = Trash) talk to it via Tauri IPC. Overlay is a static HTML bundle served over HTTP, receiving real-time state via WebSocket. i18n via shared JSON locales (ja/en/ko) loaded by both sides.

**Tech Stack:** Tauri 2.x, Rust (tokio, axum, tokio-tungstenite, reqwest, rusqlite, keyring, serde), React 18 + Vite + TypeScript, Zustand, i18next, i18n-next, `tauri-plugin-{dialog,updater,single-instance,log}`.

**Spec reference:** `docs/superpowers/specs/2026-04-24-twitch-sankagata-manager-design.md`

---

## File Structure Overview

```
twitch-sankagata-manager/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs              # App init, window spawn, panic handler
│       ├── lib.rs               # Re-exports for testability
│       ├── config.rs            # Config struct + defaults
│       ├── error.rs             # AppError enum, Result alias
│       ├── store.rs             # SQLite CRUD + migrations
│       ├── model.rs             # User, Zone, Snapshot types
│       ├── priority.rs          # Pure functions for queue ordering
│       ├── state.rs             # AppState, transitions, auto-promote
│       ├── helix.rs             # Twitch Helix REST client
│       ├── auth.rs              # OAuth device flow + keyring
│       ├── eventsub.rs          # EventSub WebSocket client
│       ├── filter.rs            # Redemption → state filter logic
│       ├── server.rs            # axum HTTP server (overlay, ws, auth cb)
│       ├── ipc.rs               # Tauri invoke handlers
│       ├── i18n.rs              # Locale loader, `t(key, args)`
│       └── overlay_assets.rs    # include_str! for overlay HTML/CSS/JS
├── src/
│   ├── main.tsx
│   ├── App.tsx                  # Main window root
│   ├── Trash.tsx                # Trash window root
│   ├── ipc.ts
│   ├── store.ts                 # Zustand store
│   ├── i18n.ts
│   ├── types.ts
│   ├── components/
│   │   ├── Header.tsx
│   │   ├── PlayingPane.tsx
│   │   ├── WaitingPane.tsx
│   │   ├── Row.tsx
│   │   └── SettingsModal.tsx
│   └── styles.css
├── overlay/
│   ├── overlay.html
│   ├── overlay.css
│   └── overlay.js
├── locales/
│   ├── ja.json
│   ├── en.json
│   └── ko.json
├── .github/workflows/release.yml
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html                   # Main window HTML
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`

- [ ] **Step 1: Scaffold with `create-tauri-app`**

Run:
```bash
bun create tauri-app twitch-sankagata-manager --template react-ts --manager bun
```

(If `twitch-sankagata-manager` dir exists already, run from parent and merge; otherwise target fresh dir and copy results in.) Verify `package.json`, `src/`, `src-tauri/` all exist.

- [ ] **Step 2: Commit scaffold**

```bash
git add -A
git commit -m "feat: scaffold tauri 2 + react ts project"
```

- [ ] **Step 3: Pin dependency versions in `package.json`**

Edit `package.json` dependencies/devDependencies to include:

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-updater": "^2.0.0",
    "@tauri-apps/plugin-single-instance": "^2.0.0",
    "@tauri-apps/plugin-log": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "zustand": "^4.5.4",
    "i18next": "^23.12.2",
    "react-i18next": "^15.0.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.3",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.1",
    "typescript": "^5.5.4",
    "vite": "^5.4.0",
    "vitest": "^2.0.5",
    "@testing-library/react": "^16.0.0",
    "@testing-library/jest-dom": "^6.4.8",
    "jsdom": "^25.0.0"
  }
}
```

Run `bun install`.

- [ ] **Step 4: Add cargo deps to `src-tauri/Cargo.toml`**

Replace `[dependencies]` block:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
tauri-plugin-updater = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-log = "2"
tokio = { version = "1.39", features = ["full"] }
tokio-tungstenite = { version = "0.23", features = ["rustls-tls-webpki-roots"] }
axum = { version = "0.7", features = ["ws"] }
tower-http = { version = "0.5", features = ["cors"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
rusqlite = { version = "0.32", features = ["bundled"] }
keyring = "3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
time = { version = "0.3", features = ["serde", "macros"] }
url = "2"
futures-util = "0.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
rand = "0.8"

[dev-dependencies]
wiremock = "0.6"
tempfile = "3"
tokio-test = "0.4"
```

- [ ] **Step 5: Configure `src-tauri/tauri.conf.json` for two windows + overlay permissions**

Replace `"windows"` array:

```json
"windows": [
  {
    "label": "main",
    "title": "参加型 Manager",
    "width": 900,
    "height": 600,
    "minWidth": 700,
    "minHeight": 450
  },
  {
    "label": "trash",
    "title": "Trash",
    "width": 320,
    "height": 500,
    "visible": false,
    "url": "trash.html"
  }
]
```

- [ ] **Step 6: Add second HTML entry for trash window**

Create `trash.html` at project root mirroring `index.html` but loading `src/Trash.tsx`:

```html
<!DOCTYPE html>
<html lang="ja">
  <head>
    <meta charset="UTF-8" />
    <title>Trash</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/trash-entry.tsx"></script>
  </body>
</html>
```

Edit `vite.config.ts`:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        trash: resolve(__dirname, "trash.html"),
      },
    },
  },
});
```

- [ ] **Step 7: Sanity-build**

Run:
```bash
bun run tauri build --debug
```
Expected: compiles cleanly, produces a debug binary. (First build may take 10+ minutes.)

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: pin deps and configure dual-window layout"
```

---

## Task 2: Shared Types + Locale JSON

**Files:**
- Create: `locales/ja.json`, `locales/en.json`, `locales/ko.json`, `src-tauri/src/model.rs`, `src-tauri/src/config.rs`, `src-tauri/src/error.rs`, `src-tauri/src/lib.rs`, `src/types.ts`

- [ ] **Step 1: Write `src-tauri/src/model.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub join_count: u32,
    pub last_join_at: Option<i64>,   // unix millis utc
    pub enqueued_at: i64,
    pub manual_order: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Zone {
    Playing,
    Waiting,
    Trash,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    #[serde(rename = "type")]
    pub kind: &'static str,            // "state"
    pub playing: Vec<User>,
    pub waiting: Vec<User>,
    pub waiting_total: usize,
    pub trash: Vec<User>,
    pub enabled: bool,
    pub language: String,
}

impl Snapshot {
    pub fn new(
        playing: Vec<User>,
        waiting: Vec<User>,
        trash: Vec<User>,
        enabled: bool,
        language: String,
    ) -> Self {
        let waiting_total = waiting.len();
        Self {
            kind: "state",
            playing,
            waiting,
            waiting_total,
            trash,
            enabled,
            language,
        }
    }
}
```

- [ ] **Step 2: Write `src-tauri/src/config.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub reward_id: Option<String>,
    pub keyword: String,
    pub max_playing: u32,
    pub max_waiting: u32,
    pub auto_promote: bool,
    pub enabled: bool,
    pub language: Language,
    pub port: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ja,
    En,
    Ko,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reward_id: None,
            keyword: "参加".to_string(),
            max_playing: 4,
            max_waiting: 3,
            auto_promote: true,
            enabled: true,
            language: Language::Ja,
            port: 24816,
        }
    }
}
```

- [ ] **Step 3: Write `src-tauri/src/error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("twitch api error: {0}")]
    Twitch(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

- [ ] **Step 4: Create `src-tauri/src/lib.rs`**

```rust
pub mod config;
pub mod error;
pub mod model;
```

Edit `main.rs` top:

```rust
use twitch_sankagata_manager_lib as app;
```

(Rename crate as needed — use `Cargo.toml` `[lib] name = "twitch_sankagata_manager_lib"`.)

- [ ] **Step 5: Write `src/types.ts` (TS mirror)**

```ts
export type Zone = "playing" | "waiting" | "trash";

export type User = {
  id: string;
  name: string;
  displayName: string;
  joinCount: number;
  lastJoinAt: number | null;
  enqueuedAt: number;
  manualOrder: number | null;
};

export type Language = "ja" | "en" | "ko";

export type Config = {
  rewardId: string | null;
  keyword: string;
  maxPlaying: number;
  maxWaiting: number;
  autoPromote: boolean;
  enabled: boolean;
  language: Language;
  port: number;
};

export type Snapshot = {
  type: "state";
  playing: User[];
  waiting: User[];
  waitingTotal: number;
  trash: User[];
  enabled: boolean;
  language: Language;
};
```

- [ ] **Step 6: Serde camelCase attribute**

Rust structs serialize snake_case by default. Add `#[serde(rename_all = "camelCase")]` on `User`, `Snapshot`, `Config` in their respective files so JSON matches TS.

- [ ] **Step 7: Write `locales/ja.json`**

```json
{
  "header": {
    "login": "Twitchでログイン",
    "loggedInAs": "ログイン中: {name}",
    "copyUrl": "OBS URLをコピー",
    "settings": "設定",
    "statusConnected": "接続済み",
    "statusReconnecting": "再接続中...",
    "statusAuthError": "認証エラー",
    "enabled": "ON",
    "disabled": "OFF"
  },
  "panes": {
    "playing": "▶ プレイ中",
    "waiting": "⏳ 待機中",
    "trash": "🗑 ゴミ箱",
    "emptySlot": "空きスロット",
    "noWaiting": "待機中のユーザーはいません"
  },
  "row": {
    "sendToTrash": "ゴミ箱へ",
    "restore": "戻す",
    "badgeTitle": "本日初回参加"
  },
  "settings": {
    "title": "設定",
    "reward": "対象リワード",
    "rewardAuto": "自動検出 (\"参加\" キーワード)",
    "keyword": "キーワード",
    "maxPlaying": "最大プレイ人数",
    "maxWaiting": "オーバーレイ表示人数",
    "autoPromote": "自動昇格",
    "language": "言語",
    "reset": "カウントをリセット",
    "resetTitle": "リセットの確認",
    "resetWarning": "すべての参加回数をリセットします。元に戻せません。",
    "resetConfirm": "リセット",
    "cancel": "キャンセル",
    "save": "保存",
    "openLogFolder": "ログフォルダを開く",
    "reAuth": "再ログイン"
  },
  "overlay": {
    "moreWaiting": "+ {n} 人待機"
  },
  "errors": {
    "noAffiliate": "チャンネルポイントを使用するにはアフィリエイト以上が必要です",
    "portInUse": "ポート {port} が使用中です。他のアプリを閉じてください",
    "reconnecting": "再接続中...",
    "dbCorrupt": "データが破損していたためリセットしました",
    "updateReady": "アップデートの準備ができました。再起動しますか?",
    "genericFatal": "予期しないエラーが発生しました"
  }
}
```

- [ ] **Step 8: Write `locales/en.json`**

```json
{
  "header": {
    "login": "Log in with Twitch",
    "loggedInAs": "Logged in as {name}",
    "copyUrl": "Copy OBS URL",
    "settings": "Settings",
    "statusConnected": "Connected",
    "statusReconnecting": "Reconnecting...",
    "statusAuthError": "Auth error",
    "enabled": "ON",
    "disabled": "OFF"
  },
  "panes": {
    "playing": "▶ Playing",
    "waiting": "⏳ Waiting",
    "trash": "🗑 Trash",
    "emptySlot": "Empty slot",
    "noWaiting": "No one waiting"
  },
  "row": {
    "sendToTrash": "Send to trash",
    "restore": "Restore",
    "badgeTitle": "First time today"
  },
  "settings": {
    "title": "Settings",
    "reward": "Target reward",
    "rewardAuto": "Auto-detect (keyword match)",
    "keyword": "Keyword",
    "maxPlaying": "Max playing",
    "maxWaiting": "Overlay visible count",
    "autoPromote": "Auto-promote",
    "language": "Language",
    "reset": "Reset counts",
    "resetTitle": "Confirm reset",
    "resetWarning": "This will reset all join counts. Cannot be undone.",
    "resetConfirm": "Reset",
    "cancel": "Cancel",
    "save": "Save",
    "openLogFolder": "Open log folder",
    "reAuth": "Re-authenticate"
  },
  "overlay": {
    "moreWaiting": "+ {n} more waiting"
  },
  "errors": {
    "noAffiliate": "Channel points require affiliate or partner status",
    "portInUse": "Port {port} is in use. Close other apps using it.",
    "reconnecting": "Reconnecting...",
    "dbCorrupt": "Data was corrupt and has been reset",
    "updateReady": "Update ready. Restart now?",
    "genericFatal": "An unexpected error occurred"
  }
}
```

- [ ] **Step 9: Write `locales/ko.json`**

```json
{
  "header": {
    "login": "Twitch로 로그인",
    "loggedInAs": "로그인: {name}",
    "copyUrl": "OBS URL 복사",
    "settings": "설정",
    "statusConnected": "연결됨",
    "statusReconnecting": "재연결 중...",
    "statusAuthError": "인증 오류",
    "enabled": "ON",
    "disabled": "OFF"
  },
  "panes": {
    "playing": "▶ 플레이 중",
    "waiting": "⏳ 대기 중",
    "trash": "🗑 휴지통",
    "emptySlot": "빈 슬롯",
    "noWaiting": "대기자가 없습니다"
  },
  "row": {
    "sendToTrash": "휴지통으로",
    "restore": "복원",
    "badgeTitle": "오늘 첫 참가"
  },
  "settings": {
    "title": "설정",
    "reward": "대상 리워드",
    "rewardAuto": "자동 감지 (키워드)",
    "keyword": "키워드",
    "maxPlaying": "최대 플레이 인원",
    "maxWaiting": "오버레이 표시 수",
    "autoPromote": "자동 승격",
    "language": "언어",
    "reset": "카운트 초기화",
    "resetTitle": "초기화 확인",
    "resetWarning": "모든 참가 횟수가 초기화됩니다. 되돌릴 수 없습니다.",
    "resetConfirm": "초기화",
    "cancel": "취소",
    "save": "저장",
    "openLogFolder": "로그 폴더 열기",
    "reAuth": "다시 로그인"
  },
  "overlay": {
    "moreWaiting": "+ {n} 명 대기"
  },
  "errors": {
    "noAffiliate": "채널 포인트를 사용하려면 제휴 이상이 필요합니다",
    "portInUse": "포트 {port}가 사용 중입니다. 다른 앱을 종료하세요.",
    "reconnecting": "재연결 중...",
    "dbCorrupt": "데이터가 손상되어 초기화되었습니다",
    "updateReady": "업데이트가 준비되었습니다. 재시작할까요?",
    "genericFatal": "예기치 않은 오류가 발생했습니다"
  }
}
```

- [ ] **Step 10: Compile check**

```bash
cd src-tauri && cargo check
```
Expected: no errors. Address any from `lib`/`main` wiring.

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: add shared types, config defaults, and locale files"
```

---

## Task 3: SQLite Store + Migrations

**Files:**
- Create: `src-tauri/src/store.rs`, `src-tauri/tests/store_test.rs`

- [ ] **Step 1: Write failing test `src-tauri/tests/store_test.rs`**

```rust
use twitch_sankagata_manager_lib::store::Store;
use twitch_sankagata_manager_lib::model::{User, Zone};
use tempfile::tempdir;

#[test]
fn opens_and_migrates_fresh_db() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = Store::open(&path).unwrap();
    assert!(path.exists());
    // initial config row present
    let cfg = store.load_config().unwrap();
    assert_eq!(cfg.keyword, "参加");
    assert_eq!(cfg.max_playing, 4);
}

#[test]
fn upserts_and_loads_user() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(),
        name: "alice".into(),
        display_name: "Alice".into(),
        join_count: 2,
        last_join_at: Some(1_700_000_000_000),
        enqueued_at: 1_700_000_000_500,
        manual_order: None,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    let loaded = store.load_zone(Zone::Waiting).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "Alice");
}

#[test]
fn moves_user_between_zones() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(), name: "a".into(), display_name: "A".into(),
        join_count: 0, last_join_at: None, enqueued_at: 1, manual_order: None,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    store.move_user("u1", Zone::Playing, 0).unwrap();
    assert!(store.load_zone(Zone::Waiting).unwrap().is_empty());
    assert_eq!(store.load_zone(Zone::Playing).unwrap().len(), 1);
}

#[test]
fn reset_counts_wipes_history_only() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(), name: "a".into(), display_name: "A".into(),
        join_count: 5, last_join_at: Some(1), enqueued_at: 10, manual_order: None,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    store.reset_counts().unwrap();
    let u = &store.load_zone(Zone::Waiting).unwrap()[0];
    assert_eq!(u.join_count, 0);
    assert_eq!(u.last_join_at, None);
}
```

- [ ] **Step 2: Run test; expect FAIL**

```bash
cd src-tauri && cargo test --test store_test
```
Expected: compile errors ("unresolved import store"). Good — proves missing module.

- [ ] **Step 3: Implement `src-tauri/src/store.rs`**

```rust
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::model::{User, Zone};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    join_count INTEGER NOT NULL DEFAULT 0,
    last_join_at INTEGER,
    enqueued_at INTEGER NOT NULL,
    manual_order INTEGER,
    zone TEXT NOT NULL,
    position INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_users_zone ON users(zone, position);

CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    blob TEXT NOT NULL
);
"#;

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        // seed default config if missing
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM config", [], |r| r.get(0))?;
        if count == 0 {
            let cfg = Config::default();
            conn.execute(
                "INSERT INTO config (id, blob) VALUES (1, ?1)",
                params![serde_json::to_string(&cfg)?],
            )?;
        }
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn load_config(&self) -> Result<Config> {
        let conn = self.conn.lock().unwrap();
        let blob: String = conn.query_row("SELECT blob FROM config WHERE id=1", [], |r| r.get(0))?;
        Ok(serde_json::from_str(&blob)?)
    }

    pub fn save_config(&self, cfg: &Config) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET blob = ?1 WHERE id = 1",
            params![serde_json::to_string(cfg)?],
        )?;
        Ok(())
    }

    pub fn upsert_user(&self, user: &User, zone: Zone, position: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO users (id,name,display_name,join_count,last_join_at,enqueued_at,manual_order,zone,position)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
            ON CONFLICT(id) DO UPDATE SET
                name=excluded.name,
                display_name=excluded.display_name,
                join_count=excluded.join_count,
                last_join_at=excluded.last_join_at,
                enqueued_at=excluded.enqueued_at,
                manual_order=excluded.manual_order,
                zone=excluded.zone,
                position=excluded.position
            "#,
            params![
                user.id, user.name, user.display_name, user.join_count,
                user.last_join_at, user.enqueued_at, user.manual_order,
                zone_to_str(zone), position
            ],
        )?;
        Ok(())
    }

    pub fn load_zone(&self, zone: Zone) -> Result<Vec<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,display_name,join_count,last_join_at,enqueued_at,manual_order \
             FROM users WHERE zone=?1 ORDER BY position ASC"
        )?;
        let rows = stmt.query_map(params![zone_to_str(zone)], |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
                display_name: r.get(2)?,
                join_count: r.get(3)?,
                last_join_at: r.get(4)?,
                enqueued_at: r.get(5)?,
                manual_order: r.get(6)?,
            })
        })?;
        let mut v = Vec::new();
        for row in rows { v.push(row?); }
        Ok(v)
    }

    pub fn move_user(&self, user_id: &str, zone: Zone, position: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE users SET zone=?1, position=?2 WHERE id=?3",
            params![zone_to_str(zone), position, user_id],
        )?;
        if rows == 0 {
            return Err(AppError::Other(format!("user {user_id} not found")));
        }
        Ok(())
    }

    pub fn delete_user(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM users WHERE id=?1", params![user_id])?;
        Ok(())
    }

    pub fn reset_counts(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET join_count=0, last_join_at=NULL, manual_order=NULL",
            [],
        )?;
        Ok(())
    }

    pub fn trim_trash(&self, cap: usize) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM users WHERE id IN (
                SELECT id FROM users WHERE zone='trash'
                ORDER BY position ASC
                LIMIT max(0, (SELECT COUNT(*) FROM users WHERE zone='trash') - ?1)
             )",
            params![cap as i64],
        )?;
        Ok(())
    }
}

fn zone_to_str(z: Zone) -> &'static str {
    match z { Zone::Playing => "playing", Zone::Waiting => "waiting", Zone::Trash => "trash" }
}
```

- [ ] **Step 4: Export `store` in `lib.rs`**

Edit `src-tauri/src/lib.rs`:

```rust
pub mod config;
pub mod error;
pub mod model;
pub mod store;
```

- [ ] **Step 5: Run tests; expect PASS**

```bash
cd src-tauri && cargo test --test store_test
```
Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(store): sqlite persistence with zones, config, reset"
```

---

## Task 4: Priority Logic (Pure Functions)

**Files:**
- Create: `src-tauri/src/priority.rs`, `src-tauri/tests/priority_test.rs`

- [ ] **Step 1: Write failing tests `src-tauri/tests/priority_test.rs`**

```rust
use twitch_sankagata_manager_lib::model::User;
use twitch_sankagata_manager_lib::priority::{is_first_time_today, sort_waiting};

fn user(id: &str, count: u32, last: Option<i64>, enq: i64, manual: Option<i64>) -> User {
    User {
        id: id.into(), name: id.into(), display_name: id.into(),
        join_count: count, last_join_at: last, enqueued_at: enq, manual_order: manual,
    }
}

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const NOW: i64 = 1_700_000_000_000;

#[test]
fn first_time_when_never_joined() {
    assert!(is_first_time_today(&user("u", 0, None, 0, None), NOW));
}

#[test]
fn first_time_when_24h_passed() {
    let past = NOW - DAY_MS - 1;
    assert!(is_first_time_today(&user("u", 3, Some(past), 0, None), NOW));
}

#[test]
fn not_first_time_when_within_24h() {
    let recent = NOW - DAY_MS + 1;
    assert!(!is_first_time_today(&user("u", 1, Some(recent), 0, None), NOW));
}

#[test]
fn sort_puts_first_timers_above_repeats() {
    let repeat = user("r", 2, Some(NOW - 1000), 100, None);
    let first = user("f", 0, None, 200, None);
    let sorted = sort_waiting(vec![repeat.clone(), first.clone()], NOW);
    assert_eq!(sorted[0].id, "f");
    assert_eq!(sorted[1].id, "r");
}

#[test]
fn sort_breaks_tie_with_enqueued_at_fifo() {
    let a = user("a", 0, None, 100, None);
    let b = user("b", 0, None, 50, None);
    let sorted = sort_waiting(vec![a, b], NOW);
    assert_eq!(sorted[0].id, "b");
    assert_eq!(sorted[1].id, "a");
}

#[test]
fn manual_order_overrides_priority() {
    let first = user("f", 0, None, 100, None);          // should be #1 by priority
    let repeat_pinned = user("r", 5, Some(NOW - 1000), 50, Some(0));  // manual #0
    let sorted = sort_waiting(vec![first, repeat_pinned], NOW);
    assert_eq!(sorted[0].id, "r");
    assert_eq!(sorted[1].id, "f");
}

#[test]
fn multiple_manual_orders_sorted_ascending() {
    let a = user("a", 0, None, 100, Some(2));
    let b = user("b", 0, None, 50, Some(0));
    let c = user("c", 0, None, 200, Some(1));
    let sorted = sort_waiting(vec![a, b, c], NOW);
    assert_eq!(sorted[0].id, "b");
    assert_eq!(sorted[1].id, "c");
    assert_eq!(sorted[2].id, "a");
}
```

- [ ] **Step 2: Run tests; expect FAIL (module missing)**

```bash
cd src-tauri && cargo test --test priority_test
```

- [ ] **Step 3: Implement `src-tauri/src/priority.rs`**

```rust
use crate::model::User;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub fn is_first_time_today(user: &User, now_ms: i64) -> bool {
    match user.last_join_at {
        None => true,
        Some(t) => (now_ms - t) > DAY_MS,
    }
}

pub fn sort_waiting(mut users: Vec<User>, now_ms: i64) -> Vec<User> {
    users.sort_by(|a, b| {
        // 1) manual order: both set => ascending; one set => it wins
        match (a.manual_order, b.manual_order) {
            (Some(x), Some(y)) => return x.cmp(&y),
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        // 2) first-time-today first
        let a_first = is_first_time_today(a, now_ms);
        let b_first = is_first_time_today(b, now_ms);
        if a_first != b_first {
            return b_first.cmp(&a_first); // true(1) > false(0)
        }
        // 3) enqueued_at ascending (FIFO)
        a.enqueued_at.cmp(&b.enqueued_at)
    });
    users
}
```

- [ ] **Step 4: Add to `lib.rs`**

```rust
pub mod priority;
```

- [ ] **Step 5: Run tests; expect PASS**

```bash
cd src-tauri && cargo test --test priority_test
```
Expected: 7 tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(priority): first-time-today + manual order sort"
```

---

## Task 5: AppState with Zone Transitions + Auto-Promote

**Files:**
- Create: `src-tauri/src/state.rs`, `src-tauri/tests/state_test.rs`

- [ ] **Step 1: Write failing tests `src-tauri/tests/state_test.rs`**

```rust
use twitch_sankagata_manager_lib::config::Config;
use twitch_sankagata_manager_lib::model::{User, Zone};
use twitch_sankagata_manager_lib::state::AppState;
use twitch_sankagata_manager_lib::store::Store;
use tempfile::tempdir;
use std::sync::Arc;

fn mk_user(id: &str) -> User {
    User {
        id: id.into(), name: id.into(), display_name: id.into(),
        join_count: 0, last_join_at: None, enqueued_at: 0, manual_order: None,
    }
}

fn new_state() -> AppState {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    AppState::new(store)
}

#[test]
fn add_redemption_goes_to_waiting() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.waiting.len(), 1);
    assert_eq!(snap.playing.len(), 0);
}

#[test]
fn auto_promote_fills_playing_slots() {
    let s = new_state();
    for i in 0..3 { s.add_redemption(mk_user(&format!("u{i}")), i as i64).unwrap(); }
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 3);
    assert_eq!(snap.waiting.len(), 0);
}

#[test]
fn auto_promote_stops_at_max_playing() {
    let s = new_state();
    for i in 0..6 { s.add_redemption(mk_user(&format!("u{i}")), i as i64).unwrap(); }
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 4);
    assert_eq!(snap.waiting.len(), 2);
}

#[test]
fn promotion_increments_join_count_and_stamps_last_join() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    let p = &snap.playing[0];
    assert_eq!(p.join_count, 1);
    assert!(p.last_join_at.is_some());
}

#[test]
fn dedupe_redemption_for_existing_user() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.add_redemption(mk_user("u1"), 2_000).unwrap();  // same id
    let snap = s.snapshot();
    assert_eq!(snap.playing.len() + snap.waiting.len(), 1);
}

#[test]
fn trash_sends_user_to_trash_and_auto_promotes_waiting() {
    let s = new_state();
    for i in 0..5 { s.add_redemption(mk_user(&format!("u{i}")), i as i64).unwrap(); }
    // playing=[u0..u3], waiting=[u4]
    s.trash_user("u0").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 4);
    assert_eq!(snap.waiting.len(), 0);
    assert_eq!(snap.trash.len(), 1);
    assert!(snap.playing.iter().any(|u| u.id == "u4"));
}

#[test]
fn restore_user_returns_to_waiting_end() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.trash_user("u1").unwrap();
    s.restore_user("u1").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.trash.len(), 0);
    assert!(snap.playing.iter().any(|u| u.id == "u1") || snap.waiting.iter().any(|u| u.id == "u1"));
}

#[test]
fn refund_removes_user_and_decrements_count() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    // user is now in playing with join_count=1
    s.refund_user("u1").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 0);
    assert_eq!(snap.waiting.len(), 0);
    // dedupe check: next add should treat as fresh
    s.add_redemption(mk_user("u1"), 2_000).unwrap();
    let snap2 = s.snapshot();
    let u = snap2.playing.iter().find(|u| u.id == "u1").unwrap();
    assert_eq!(u.join_count, 1);  // back to 1 after fresh add
}

#[test]
fn disabled_skips_adds_but_keeps_queue() {
    let mut cfg = Config::default();
    cfg.enabled = false;
    let s = new_state();
    s.set_config(cfg).unwrap();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 0);
    assert_eq!(snap.waiting.len(), 0);
}

#[test]
fn move_user_updates_zone_and_sets_manual_order() {
    let s = new_state();
    for i in 0..5 { s.add_redemption(mk_user(&format!("u{i}")), i as i64).unwrap(); }
    // move u4 from waiting to playing[0] (which needs someone bumped)
    s.move_user("u4", Zone::Waiting, 0).unwrap();
    let snap = s.snapshot();
    let idx = snap.waiting.iter().position(|u| u.id == "u4").unwrap();
    assert_eq!(idx, 0);
}

#[test]
fn reset_counts_zeroes_history() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.reset_counts().unwrap();
    let snap = s.snapshot();
    let u = snap.playing.iter().chain(snap.waiting.iter()).find(|u| u.id == "u1").unwrap();
    assert_eq!(u.join_count, 0);
    assert_eq!(u.last_join_at, None);
}
```

- [ ] **Step 2: Run; expect FAIL**

```bash
cd src-tauri && cargo test --test state_test
```

- [ ] **Step 3: Implement `src-tauri/src/state.rs`**

```rust
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::model::{Snapshot, User, Zone};
use crate::priority::sort_waiting;
use crate::store::Store;
use std::sync::{Arc, Mutex};

const TRASH_CAP: usize = 200;

pub struct AppState {
    inner: Mutex<Inner>,
    store: Arc<Store>,
}

struct Inner {
    playing: Vec<User>,
    waiting: Vec<User>,
    trash: Vec<User>,
    config: Config,
}

impl AppState {
    pub fn new(store: Arc<Store>) -> Self {
        let config = store.load_config().unwrap_or_default();
        let playing = store.load_zone(Zone::Playing).unwrap_or_default();
        let waiting = store.load_zone(Zone::Waiting).unwrap_or_default();
        let trash = store.load_zone(Zone::Trash).unwrap_or_default();
        Self {
            inner: Mutex::new(Inner { playing, waiting, trash, config }),
            store,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let g = self.inner.lock().unwrap();
        Snapshot::new(
            g.playing.clone(),
            g.waiting.clone(),
            g.trash.clone(),
            g.config.enabled,
            match g.config.language {
                crate::config::Language::Ja => "ja",
                crate::config::Language::En => "en",
                crate::config::Language::Ko => "ko",
            }.into(),
        )
    }

    pub fn config(&self) -> Config { self.inner.lock().unwrap().config.clone() }

    pub fn set_config(&self, cfg: Config) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        g.config = cfg.clone();
        drop(g);
        self.store.save_config(&cfg)?;
        self.auto_promote();
        Ok(())
    }

    pub fn add_redemption(&self, user: User, now_ms: i64) -> Result<()> {
        let _ = now_ms;
        let mut g = self.inner.lock().unwrap();
        if !g.config.enabled { return Ok(()); }
        if g.playing.iter().any(|u| u.id == user.id)
            || g.waiting.iter().any(|u| u.id == user.id)
            || g.trash.iter().any(|u| u.id == user.id)
        {
            return Ok(());
        }
        let mut u = user;
        u.enqueued_at = now_ms;
        g.waiting.push(u);
        g.waiting = sort_waiting(std::mem::take(&mut g.waiting), now_ms);
        drop(g);
        self.auto_promote();
        self.persist_all()
    }

    fn auto_promote(&self) {
        let mut g = self.inner.lock().unwrap();
        if !g.config.auto_promote { return; }
        while (g.playing.len() as u32) < g.config.max_playing && !g.waiting.is_empty() {
            let mut promoted = g.waiting.remove(0);
            promoted.join_count += 1;
            promoted.last_join_at = Some(now_ms());
            g.playing.push(promoted);
        }
    }

    pub fn trash_user(&self, user_id: &str) -> Result<()> {
        {
            let mut g = self.inner.lock().unwrap();
            if let Some(pos) = g.playing.iter().position(|u| u.id == user_id) {
                let u = g.playing.remove(pos);
                g.trash.insert(0, u);
            } else if let Some(pos) = g.waiting.iter().position(|u| u.id == user_id) {
                let u = g.waiting.remove(pos);
                g.trash.insert(0, u);
            } else {
                return Err(AppError::Other(format!("user {user_id} not found")));
            }
            if g.trash.len() > TRASH_CAP { g.trash.truncate(TRASH_CAP); }
        }
        self.auto_promote();
        self.persist_all()
    }

    pub fn restore_user(&self, user_id: &str) -> Result<()> {
        {
            let mut g = self.inner.lock().unwrap();
            let pos = g.trash.iter().position(|u| u.id == user_id)
                .ok_or_else(|| AppError::Other(format!("user {user_id} not in trash")))?;
            let u = g.trash.remove(pos);
            g.waiting.push(u);
        }
        self.auto_promote();
        self.persist_all()
    }

    pub fn refund_user(&self, user_id: &str) -> Result<()> {
        {
            let mut g = self.inner.lock().unwrap();
            // remove from playing or waiting; leave trash alone per spec
            if let Some(pos) = g.playing.iter().position(|u| u.id == user_id) {
                let mut u = g.playing.remove(pos);
                if u.join_count > 0 { u.join_count -= 1; }
                // fully drop reference — no re-queue on refund
                self.store.delete_user(&u.id).ok();
            } else if let Some(pos) = g.waiting.iter().position(|u| u.id == user_id) {
                let u = g.waiting.remove(pos);
                self.store.delete_user(&u.id).ok();
            }
        }
        self.auto_promote();
        self.persist_all()
    }

    pub fn move_user(&self, user_id: &str, zone: Zone, index: usize) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        let removed = [Zone::Playing, Zone::Waiting, Zone::Trash].iter().find_map(|z| {
            let vec = match z {
                Zone::Playing => &mut g.playing,
                Zone::Waiting => &mut g.waiting,
                Zone::Trash => &mut g.trash,
            };
            if let Some(pos) = vec.iter().position(|u| u.id == user_id) {
                Some((*z, vec.remove(pos)))
            } else { None }
        });
        let Some((_from, mut user)) = removed else {
            return Err(AppError::Other(format!("user {user_id} not found")));
        };
        // mark manual order so sort respects it
        user.manual_order = Some(index as i64);
        let dest = match zone {
            Zone::Playing => &mut g.playing,
            Zone::Waiting => &mut g.waiting,
            Zone::Trash => &mut g.trash,
        };
        let idx = index.min(dest.len());
        dest.insert(idx, user);
        drop(g);
        self.auto_promote();
        self.persist_all()
    }

    pub fn reset_counts(&self) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        for u in g.playing.iter_mut().chain(g.waiting.iter_mut()).chain(g.trash.iter_mut()) {
            u.join_count = 0;
            u.last_join_at = None;
            u.manual_order = None;
        }
        drop(g);
        self.store.reset_counts()?;
        Ok(())
    }

    fn persist_all(&self) -> Result<()> {
        let g = self.inner.lock().unwrap();
        // re-upsert all users with new positions
        for (i, u) in g.playing.iter().enumerate() {
            self.store.upsert_user(u, Zone::Playing, i as i64)?;
        }
        for (i, u) in g.waiting.iter().enumerate() {
            self.store.upsert_user(u, Zone::Waiting, i as i64)?;
        }
        for (i, u) in g.trash.iter().enumerate() {
            self.store.upsert_user(u, Zone::Trash, i as i64)?;
        }
        self.store.trim_trash(TRASH_CAP)?;
        Ok(())
    }
}

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}
```

- [ ] **Step 4: Add to `lib.rs`**

```rust
pub mod state;
```

- [ ] **Step 5: Run tests; expect PASS**

```bash
cd src-tauri && cargo test --test state_test
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(state): zone transitions, auto-promote, refund handling"
```

---

## Task 6: Helix Client (Rewards + Refund)

**Files:**
- Create: `src-tauri/src/helix.rs`, `src-tauri/tests/helix_test.rs`

- [ ] **Step 1: Failing test `src-tauri/tests/helix_test.rs`**

```rust
use twitch_sankagata_manager_lib::helix::HelixClient;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use serde_json::json;

#[tokio::test]
async fn list_rewards_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/helix/channel_points/custom_rewards"))
        .and(header("Authorization", "Bearer TEST_TOKEN"))
        .and(header("Client-Id", "TEST_CLIENT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "id": "r1", "title": "参加する", "cost": 500 },
                { "id": "r2", "title": "other",    "cost": 100 }
            ]
        })))
        .mount(&server).await;

    let c = HelixClient::new(server.uri(), "TEST_CLIENT", "TEST_TOKEN");
    let rewards = c.list_rewards("12345").await.unwrap();
    assert_eq!(rewards.len(), 2);
    assert_eq!(rewards[0].title, "参加する");
}

#[tokio::test]
async fn refund_redemption_sends_patch() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/helix/channel_points/custom_rewards/redemptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [] })))
        .mount(&server).await;

    let c = HelixClient::new(server.uri(), "CID", "TOK");
    c.refund_redemption("42", "reward1", "redeem1").await.unwrap();
}
```

- [ ] **Step 2: Run; expect FAIL**

```bash
cd src-tauri && cargo test --test helix_test
```

- [ ] **Step 3: Implement `src-tauri/src/helix.rs`**

```rust
use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reward {
    pub id: String,
    pub title: String,
    pub cost: u32,
}

#[derive(Clone)]
pub struct HelixClient {
    base: String,
    client_id: String,
    token: String,
    http: Client,
}

impl HelixClient {
    pub fn new(base: impl Into<String>, client_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            client_id: client_id.into(),
            token: token.into(),
            http: Client::new(),
        }
    }

    pub async fn list_rewards(&self, broadcaster_id: &str) -> Result<Vec<Reward>> {
        #[derive(Deserialize)]
        struct Resp { data: Vec<Reward> }
        let url = format!("{}/helix/channel_points/custom_rewards", self.base);
        let resp = self.http.get(&url)
            .query(&[("broadcaster_id", broadcaster_id)])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-Id", &self.client_id)
            .send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Twitch(format!("list_rewards {status}: {body}")));
        }
        Ok(resp.json::<Resp>().await?.data)
    }

    pub async fn refund_redemption(&self, broadcaster_id: &str, reward_id: &str, redemption_id: &str) -> Result<()> {
        let url = format!("{}/helix/channel_points/custom_rewards/redemptions", self.base);
        let resp = self.http.patch(&url)
            .query(&[
                ("broadcaster_id", broadcaster_id),
                ("reward_id", reward_id),
                ("id", redemption_id),
            ])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-Id", &self.client_id)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"CANCELED"}"#)
            .send().await?;
        if !resp.status().is_success() {
            return Err(AppError::Twitch(format!("refund {}: {}", resp.status(), resp.text().await.unwrap_or_default())));
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Add to `lib.rs`**

```rust
pub mod helix;
```

- [ ] **Step 5: Run; expect PASS**

```bash
cd src-tauri && cargo test --test helix_test
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(helix): list_rewards + refund_redemption client"
```

---

## Task 7: OAuth Device-Code Flow + Keyring

**Files:**
- Create: `src-tauri/src/auth.rs`, `src-tauri/tests/auth_test.rs`

- [ ] **Step 1: Failing test (device-code flow against mock)**

```rust
use twitch_sankagata_manager_lib::auth::{DeviceFlow, StoredTokens};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;

#[tokio::test]
async fn device_flow_polls_until_token_ready() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/oauth2/device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_code": "dc", "user_code": "ABCD", "verification_uri": "https://twitch.tv/activate",
            "expires_in": 1800, "interval": 0
        })))
        .mount(&server).await;
    // first poll: pending, second: success
    Mock::given(method("POST")).and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "A", "refresh_token": "R", "expires_in": 14400
        })))
        .mount(&server).await;

    let flow = DeviceFlow::new(server.uri(), "CID", vec!["channel:read:redemptions".into()]);
    let start = flow.start().await.unwrap();
    assert_eq!(start.user_code, "ABCD");
    let tokens: StoredTokens = flow.poll(&start.device_code).await.unwrap();
    assert_eq!(tokens.access_token, "A");
}

#[tokio::test]
async fn refresh_token_obtains_new_access() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "NEW", "refresh_token": "NEW_R", "expires_in": 14400
        })))
        .mount(&server).await;
    let flow = DeviceFlow::new(server.uri(), "CID", vec![]);
    let t = flow.refresh("OLD_R").await.unwrap();
    assert_eq!(t.access_token, "NEW");
}
```

- [ ] **Step 2: Run; expect FAIL**

- [ ] **Step 3: Implement `src-tauri/src/auth.rs`**

```rust
use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
        Self { base: base.into(), client_id: client_id.into(), scopes, http: Client::new() }
    }

    pub async fn start(&self) -> Result<DeviceStart> {
        let url = format!("{}/oauth2/device", self.base);
        let scope_str = self.scopes.join(" ");
        let resp = self.http.post(&url)
            .form(&[("client_id", self.client_id.as_str()), ("scopes", scope_str.as_str())])
            .send().await?;
        if !resp.status().is_success() {
            return Err(AppError::Auth(format!("device start {}", resp.status())));
        }
        Ok(resp.json().await?)
    }

    pub async fn poll(&self, device_code: &str) -> Result<StoredTokens> {
        let url = format!("{}/oauth2/token", self.base);
        loop {
            let resp = self.http.post(&url)
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send().await?;
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
        let resp = self.http.post(&url)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send().await?;
        if !resp.status().is_success() {
            return Err(AppError::Auth(format!("refresh {}", resp.status())));
        }
        Ok(resp.json().await?)
    }
}

pub fn store_tokens(tokens: &StoredTokens) -> Result<()> {
    let entry = keyring::Entry::new("twitch-sankagata-manager", "twitch")?;
    entry.set_password(&serde_json::to_string(tokens)?)?;
    Ok(())
}

pub fn load_tokens() -> Result<Option<StoredTokens>> {
    let entry = keyring::Entry::new("twitch-sankagata-manager", "twitch")?;
    match entry.get_password() {
        Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e)),
    }
}

pub fn clear_tokens() -> Result<()> {
    let entry = keyring::Entry::new("twitch-sankagata-manager", "twitch")?;
    entry.delete_credential().ok();
    Ok(())
}
```

- [ ] **Step 4: Register in `lib.rs`**

```rust
pub mod auth;
```

- [ ] **Step 5: Run tests (only flow tests — keyring calls not exercised)**

```bash
cd src-tauri && cargo test --test auth_test
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(auth): twitch oauth device flow + keyring storage"
```

---

## Task 8: EventSub WebSocket Client + Reconnect

**Files:**
- Create: `src-tauri/src/eventsub.rs`, `src-tauri/tests/eventsub_test.rs`

- [ ] **Step 1: Failing test (run a local WS server, feed canned Twitch frames)**

```rust
use twitch_sankagata_manager_lib::eventsub::{EventSubClient, EventSubMessage};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

#[tokio::test]
async fn receives_session_welcome_and_notification() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"metadata":{"message_type":"session_welcome"},"payload":{"session":{"id":"S1","status":"connected","keepalive_timeout_seconds":10}}}"#.into())).await.unwrap();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"metadata":{"message_type":"notification","subscription_type":"channel.channel_points_custom_reward_redemption.add"},"payload":{"event":{"user_id":"u1","user_name":"alice","user_login":"alice","reward":{"id":"r1","title":"参加する"},"id":"rd1","status":"UNFULFILLED"}}}"#.into())).await.unwrap();
    });
    let url = format!("ws://{addr}");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let client = EventSubClient::new(url, tx);
    tokio::spawn(async move { client.run().await.ok(); });

    // first message should be session_welcome
    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
    assert!(matches!(msg, EventSubMessage::Welcome { .. }));
    let msg2 = rx.recv().await.unwrap();
    assert!(matches!(msg2, EventSubMessage::Notification { .. }));

    server.await.ok();
}
```

- [ ] **Step 2: Run; expect FAIL (compile: module missing)**

- [ ] **Step 3: Implement `src-tauri/src/eventsub.rs`**

```rust
use crate::error::{AppError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;

pub const PROD_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

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
pub struct RewardRef { pub id: String, pub title: String }

#[derive(Debug, Clone)]
pub enum EventSubMessage {
    Welcome { session_id: String, reconnect_url: Option<String> },
    Notification { subscription_type: String, event: serde_json::Value },
    Keepalive,
    Reconnect { new_url: String },
    Revocation,
}

pub struct EventSubClient {
    url: String,
    tx: UnboundedSender<EventSubMessage>,
}

impl EventSubClient {
    pub fn new(url: impl Into<String>, tx: UnboundedSender<EventSubMessage>) -> Self {
        Self { url: url.into(), tx }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut backoff_secs: u64 = 1;
        loop {
            match self.once().await {
                Ok(Some(new_url)) => {
                    self.url = new_url;
                    backoff_secs = 1;
                    continue;
                }
                Ok(None) => return Ok(()),
                Err(e) => {
                    tracing::warn!("eventsub disconnected: {e}, reconnecting in {backoff_secs}s");
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(60);
                }
            }
        }
    }

    async fn once(&self) -> Result<Option<String>> {
        let (mut ws, _) = connect_async(&self.url).await
            .map_err(|e| AppError::Other(e.to_string()))?;
        while let Some(msg) = ws.next().await {
            let msg = msg.map_err(|e| AppError::Other(e.to_string()))?;
            if let tokio_tungstenite::tungstenite::Message::Text(t) = msg {
                let v: serde_json::Value = serde_json::from_str(&t)?;
                let mtype = v["metadata"]["message_type"].as_str().unwrap_or("");
                match mtype {
                    "session_welcome" => {
                        let sid = v["payload"]["session"]["id"].as_str().unwrap_or("").to_string();
                        let _ = self.tx.send(EventSubMessage::Welcome { session_id: sid, reconnect_url: None });
                    }
                    "session_keepalive" => { let _ = self.tx.send(EventSubMessage::Keepalive); }
                    "notification" => {
                        let stype = v["metadata"]["subscription_type"].as_str().unwrap_or("").to_string();
                        let event = v["payload"]["event"].clone();
                        let _ = self.tx.send(EventSubMessage::Notification { subscription_type: stype, event });
                    }
                    "session_reconnect" => {
                        let url = v["payload"]["session"]["reconnect_url"].as_str().unwrap_or("").to_string();
                        let _ = self.tx.send(EventSubMessage::Reconnect { new_url: url.clone() });
                        ws.close(None).await.ok();
                        return Ok(Some(url));
                    }
                    "revocation" => { let _ = self.tx.send(EventSubMessage::Revocation); }
                    _ => {}
                }
            }
        }
        Err(AppError::Other("websocket closed".into()))
    }
}

pub async fn subscribe(
    helix_base: &str, client_id: &str, token: &str,
    session_id: &str, broadcaster_id: &str, event_type: &str,
) -> Result<()> {
    let http = reqwest::Client::new();
    let body = serde_json::json!({
        "type": event_type,
        "version": "1",
        "condition": { "broadcaster_user_id": broadcaster_id },
        "transport": { "method": "websocket", "session_id": session_id },
    });
    let resp = http.post(format!("{helix_base}/helix/eventsub/subscriptions"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Client-Id", client_id)
        .json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Twitch(format!("subscribe {}: {}", resp.status(), resp.text().await.unwrap_or_default())));
    }
    Ok(())
}
```

- [ ] **Step 4: Register module**

```rust
// lib.rs
pub mod eventsub;
```

- [ ] **Step 5: Run tests; expect PASS**

```bash
cd src-tauri && cargo test --test eventsub_test
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(eventsub): websocket client with reconnect + subscribe helper"
```

---

## Task 9: Filter Logic (Redemption → State)

**Files:**
- Create: `src-tauri/src/filter.rs`, `src-tauri/tests/filter_test.rs`

- [ ] **Step 1: Failing tests**

```rust
use twitch_sankagata_manager_lib::config::Config;
use twitch_sankagata_manager_lib::filter::{should_accept, ParsedRedemption};
use twitch_sankagata_manager_lib::eventsub::{Redemption, RewardRef};

fn red(reward_id: &str, title: &str) -> Redemption {
    Redemption {
        id: "rd1".into(), user_id: "u1".into(), user_name: "A".into(), user_login: "a".into(),
        reward: RewardRef { id: reward_id.into(), title: title.into() },
        status: "UNFULFILLED".into(),
    }
}

#[test]
fn accepts_when_reward_id_matches() {
    let mut cfg = Config::default();
    cfg.reward_id = Some("target".into());
    assert!(should_accept(&red("target", "anything"), &cfg));
    assert!(!should_accept(&red("other", "anything"), &cfg));
}

#[test]
fn falls_back_to_keyword_when_no_reward_id() {
    let mut cfg = Config::default();
    cfg.reward_id = None;
    cfg.keyword = "参加".into();
    assert!(should_accept(&red("x", "参加する"), &cfg));
    assert!(!should_accept(&red("x", "ゲーム無関係"), &cfg));
}

#[test]
fn reward_id_wins_over_keyword() {
    let mut cfg = Config::default();
    cfg.reward_id = Some("target".into());
    cfg.keyword = "参加".into();
    // title contains keyword but id does not match -> reject
    assert!(!should_accept(&red("other", "参加!"), &cfg));
}

#[test]
fn parsed_redemption_maps_fields() {
    let r = red("r", "t");
    let p = ParsedRedemption::from(&r);
    assert_eq!(p.user.id, "u1");
    assert_eq!(p.user.display_name, "A");
    assert_eq!(p.redemption_id, "rd1");
}
```

- [ ] **Step 2: Run; expect FAIL**

- [ ] **Step 3: Implement `src-tauri/src/filter.rs`**

```rust
use crate::config::Config;
use crate::eventsub::Redemption;
use crate::model::User;

pub fn should_accept(r: &Redemption, cfg: &Config) -> bool {
    match cfg.reward_id.as_deref() {
        Some(target) => r.reward.id == target,
        None => r.reward.title.contains(&cfg.keyword),
    }
}

pub struct ParsedRedemption {
    pub user: User,
    pub redemption_id: String,
    pub reward_id: String,
}

impl From<&Redemption> for ParsedRedemption {
    fn from(r: &Redemption) -> Self {
        Self {
            user: User {
                id: r.user_id.clone(),
                name: r.user_login.clone(),
                display_name: r.user_name.clone(),
                join_count: 0,
                last_join_at: None,
                enqueued_at: 0,
                manual_order: None,
            },
            redemption_id: r.id.clone(),
            reward_id: r.reward.id.clone(),
        }
    }
}
```

- [ ] **Step 4: Register module**

```rust
// lib.rs
pub mod filter;
```

- [ ] **Step 5: Run tests; expect PASS**

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(filter): redemption accept logic + parse to User"
```

---

## Task 10: HTTP Server (axum) with Overlay + WebSocket

**Files:**
- Create: `src-tauri/src/server.rs`, `src-tauri/src/overlay_assets.rs`, `overlay/overlay.html`, `overlay/overlay.css`, `overlay/overlay.js`, `src-tauri/tests/server_test.rs`

- [ ] **Step 1: Write overlay HTML `overlay/overlay.html`**

```html
<!DOCTYPE html>
<html><head>
<meta charset="UTF-8"><title>Overlay</title>
<link rel="stylesheet" href="overlay.css">
</head><body><div id="root"></div>
<script src="overlay.js"></script></body></html>
```

- [ ] **Step 2: Write `overlay/overlay.css`**

```css
html, body { margin: 0; padding: 0; background: transparent; font-family: system-ui, sans-serif; color: #fff; }
#root { padding: 10px; }
.row { display: flex; align-items: center; gap: 8px; padding: 7px 12px; border-radius: 5px;
       background: rgba(20, 22, 30, 0.35); margin: 4px 0; text-shadow: 1px 1px 2px rgba(0,0,0,0.9); font-size: 14px; }
.row.playing { border-left: 3px solid #6af; background: rgba(20, 22, 30, 0.4); }
.row.waiting { background: rgba(20, 22, 30, 0.3); font-size: 12px; opacity: 0.85; }
.name { font-family: monospace; font-weight: 500; flex: 1; }
.badge { background: #fc3; color: #000; font-weight: 800; font-size: 10px; padding: 2px 6px; border-radius: 3px;
         box-shadow: 0 1px 3px rgba(0,0,0,0.4); }
.more { text-align: center; font-size: 11px; opacity: 0.7; padding: 3px; font-style: italic; }
```

- [ ] **Step 3: Write `overlay/overlay.js`**

```js
(function () {
  const DAY_MS = 24 * 60 * 60 * 1000;
  const root = document.getElementById("root");
  const moreTmpl = { ja: "+ {n} 人待機", en: "+ {n} more waiting", ko: "+ {n} 명 대기" };

  function isFirstToday(u) {
    return u.lastJoinAt === null || (Date.now() - u.lastJoinAt) > DAY_MS;
  }

  function render(snap) {
    const lang = snap.language || "ja";
    const parts = [];
    for (const u of snap.playing) {
      parts.push(`<div class="row playing"><span class="name">${escape(u.displayName)}</span>${isFirstToday(u) ? '<span class="badge">初</span>' : ''}</div>`);
    }
    const visible = snap.waiting.slice(0, snap.playing.length ? 99 : 99); // overlay always shows all visible waiting passed from backend
    for (const u of visible) {
      parts.push(`<div class="row waiting"><span class="name">${escape(u.displayName)}</span>${isFirstToday(u) ? '<span class="badge">初</span>' : ''}</div>`);
    }
    const hidden = snap.waitingTotal - visible.length;
    if (hidden > 0) {
      parts.push(`<div class="more">${(moreTmpl[lang] || moreTmpl.ja).replace("{n}", hidden)}</div>`);
    }
    root.innerHTML = parts.join("");
  }

  function escape(s) { return String(s).replace(/[&<>"']/g, c => ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;" }[c])); }

  function connect() {
    const ws = new WebSocket(`ws://${location.host}/ws`);
    ws.onmessage = (e) => {
      try { render(JSON.parse(e.data)); } catch (err) { console.error(err); }
    };
    ws.onclose = () => setTimeout(connect, 1000);
  }
  connect();
})();
```

- [ ] **Step 4: Create `src-tauri/src/overlay_assets.rs`**

```rust
pub const OVERLAY_HTML: &str = include_str!("../../overlay/overlay.html");
pub const OVERLAY_CSS: &str = include_str!("../../overlay/overlay.css");
pub const OVERLAY_JS: &str = include_str!("../../overlay/overlay.js");
```

- [ ] **Step 5: Failing test `src-tauri/tests/server_test.rs`**

```rust
use twitch_sankagata_manager_lib::server::build_router;
use twitch_sankagata_manager_lib::state::AppState;
use twitch_sankagata_manager_lib::store::Store;
use std::sync::Arc;
use tempfile::tempdir;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn test_state() -> Arc<AppState> {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    Arc::new(AppState::new(store))
}

#[tokio::test]
async fn serves_overlay_html() {
    let router = build_router(test_state());
    let resp = router.oneshot(Request::builder().uri("/overlay").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("<html"));
}

#[tokio::test]
async fn serves_overlay_css_and_js() {
    let router = build_router(test_state());
    let css = router.clone().oneshot(Request::builder().uri("/overlay.css").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(css.status(), StatusCode::OK);
    let js = router.oneshot(Request::builder().uri("/overlay.js").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(js.status(), StatusCode::OK);
}
```

- [ ] **Step 6: Run; expect FAIL**

- [ ] **Step 7: Implement `src-tauri/src/server.rs`**

```rust
use crate::model::Snapshot;
use crate::overlay_assets::*;
use crate::state::AppState;
use axum::{
    extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade},
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

pub fn build_router(state: Arc<AppState>) -> Router {
    let (tx, _rx) = broadcast::channel::<Snapshot>(64);
    let ctx = AppCtx { state, tx };
    Router::new()
        .route("/overlay", get(overlay_html))
        .route("/overlay.css", get(overlay_css))
        .route("/overlay.js", get(overlay_js))
        .route("/ws", get(ws_handler))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(ctx)
}

async fn overlay_html() -> Response {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], OVERLAY_HTML).into_response()
}
async fn overlay_css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], OVERLAY_CSS).into_response()
}
async fn overlay_js() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript; charset=utf-8")], OVERLAY_JS).into_response()
}

async fn ws_handler(State(ctx): State<AppCtx>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |sock| handle_socket(sock, ctx))
}

async fn handle_socket(mut sock: WebSocket, ctx: AppCtx) {
    // push initial snapshot
    let snap = ctx.state.snapshot();
    let _ = sock.send(Message::Text(serde_json::to_string(&snap).unwrap_or_default())).await;

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
                    Some(Ok(_)) => continue, // ignore client messages
                    _ => break,
                }
            }
        }
    }
}

pub async fn bind_with_fallback(start_port: u16) -> std::io::Result<(tokio::net::TcpListener, u16)> {
    for p in start_port..=start_port + 10 {
        match tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
            Ok(l) => return Ok((l, p)),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "no free port in range"))
}
```

- [ ] **Step 8: Register module + compile**

```rust
// lib.rs
pub mod overlay_assets;
pub mod server;
```

Run:
```bash
cd src-tauri && cargo test --test server_test
```
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(server): axum http server with overlay assets and ws"
```

---

## Task 11: Backend Event Loop (EventSub ↔ State ↔ WS Broadcast)

**Files:**
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src-tauri/src/pipeline.rs`

- [ ] **Step 1: Write `src-tauri/src/pipeline.rs`**

```rust
use crate::eventsub::{EventSubMessage, Redemption};
use crate::filter::{should_accept, ParsedRedemption};
use crate::model::Snapshot;
use crate::state::{now_ms, AppState};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub async fn run_pipeline(
    state: Arc<AppState>,
    mut rx: mpsc::UnboundedReceiver<EventSubMessage>,
    tx: broadcast::Sender<Snapshot>,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            EventSubMessage::Notification { subscription_type, event } => {
                if subscription_type == "channel.channel_points_custom_reward_redemption.add" {
                    if let Ok(r) = serde_json::from_value::<Redemption>(event) {
                        let cfg = state.config();
                        if !should_accept(&r, &cfg) { continue; }
                        let parsed = ParsedRedemption::from(&r);
                        let _ = state.add_redemption(parsed.user, now_ms());
                        let _ = tx.send(state.snapshot());
                    }
                } else if subscription_type == "channel.channel_points_custom_reward_redemption.update" {
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
```

- [ ] **Step 2: Register module**

```rust
// lib.rs
pub mod pipeline;
```

- [ ] **Step 3: Compile check**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(pipeline): wire eventsub notifications into state + broadcast"
```

---

## Task 12: Tauri IPC Commands

**Files:**
- Create: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write `src-tauri/src/ipc.rs`**

```rust
use crate::config::Config;
use crate::error::AppError;
use crate::helix::HelixClient;
use crate::model::{Snapshot, Zone};
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct IpcError { message: String }
impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self { Self { message: e.to_string() } }
}

pub type Ctx<'a> = State<'a, Arc<AppState>>;

#[tauri::command]
pub fn get_snapshot(state: Ctx<'_>) -> Snapshot { state.snapshot() }

#[tauri::command]
pub fn get_config(state: Ctx<'_>) -> Config { state.config() }

#[tauri::command]
pub fn set_config(state: Ctx<'_>, cfg: Config) -> Result<Snapshot, IpcError> {
    state.set_config(cfg)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn set_enabled(state: Ctx<'_>, enabled: bool) -> Result<Snapshot, IpcError> {
    let mut cfg = state.config();
    cfg.enabled = enabled;
    state.set_config(cfg)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn trash_user(state: Ctx<'_>, user_id: String) -> Result<Snapshot, IpcError> {
    state.trash_user(&user_id)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn restore_user(state: Ctx<'_>, user_id: String) -> Result<Snapshot, IpcError> {
    state.restore_user(&user_id)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn move_user(state: Ctx<'_>, user_id: String, zone: Zone, index: usize) -> Result<Snapshot, IpcError> {
    state.move_user(&user_id, zone, index)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn reset_counts(state: Ctx<'_>) -> Result<Snapshot, IpcError> {
    state.reset_counts()?;
    Ok(state.snapshot())
}

#[tauri::command]
pub async fn list_rewards(
    client: State<'_, Option<(HelixClient, String)>>,
) -> Result<Vec<crate::helix::Reward>, IpcError> {
    match client.inner() {
        Some((c, broadcaster_id)) => c.list_rewards(broadcaster_id).await.map_err(Into::into),
        None => Err(IpcError { message: "not authenticated".into() }),
    }
}
```

- [ ] **Step 2: Wire into `main.rs` (see Task 13 for full init)**

- [ ] **Step 3: Commit scaffolding**

```bash
git add -A
git commit -m "feat(ipc): tauri commands for snapshot, config, zone mutations"
```

---

## Task 13: Main Entry Wiring (Tauri + Server + Pipeline)

**Files:**
- Rewrite: `src-tauri/src/main.rs`

- [ ] **Step 1: Replace `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use twitch_sankagata_manager_lib as app;
use app::{auth, eventsub, filter, pipeline, server, state::AppState, store::Store};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{broadcast, mpsc};

pub const TWITCH_CLIENT_ID: &str = env!("TWITCH_CLIENT_ID");
pub const TWITCH_API_BASE: &str = "https://api.twitch.tv";
pub const TWITCH_ID_BASE: &str = "https://id.twitch.tv";

fn main() {
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into())
    ).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") { let _ = w.set_focus(); }
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = handle.path().app_data_dir()
                .expect("app data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("state.db");
            let store = Arc::new(Store::open(&db_path).expect("open db"));
            let state = Arc::new(AppState::new(store));
            app.manage(state.clone());

            let (tx_evt, rx_evt) = mpsc::unbounded_channel();
            let (tx_snap, _rx) = broadcast::channel::<app::model::Snapshot>(64);
            app.manage(tx_snap.clone());

            // emit snapshots to all webviews on state changes
            let app_handle_clone = handle.clone();
            let mut snap_rx = tx_snap.subscribe();
            tauri::async_runtime::spawn(async move {
                while let Ok(snap) = snap_rx.recv().await {
                    let _ = app_handle_clone.emit("state-changed", &snap);
                }
            });

            // http server
            let state_for_server = state.clone();
            tauri::async_runtime::spawn(async move {
                let router = server::build_router(state_for_server);
                let port = app::config::Config::default().port;
                match server::bind_with_fallback(port).await {
                    Ok((listener, actual)) => {
                        tracing::info!("overlay at http://localhost:{actual}/overlay");
                        let _ = axum::serve(listener, router).await;
                    }
                    Err(e) => tracing::error!("server failed: {e}"),
                }
            });

            // pipeline
            let state_for_pipeline = state.clone();
            let tx_snap_for_pipeline = tx_snap.clone();
            tauri::async_runtime::spawn(async move {
                pipeline::run_pipeline(state_for_pipeline, rx_evt, tx_snap_for_pipeline).await;
            });

            // eventsub (only start if tokens exist)
            if let Ok(Some(_tokens)) = auth::load_tokens() {
                let tx_evt_clone = tx_evt.clone();
                tauri::async_runtime::spawn(async move {
                    let client = eventsub::EventSubClient::new(eventsub::PROD_URL, tx_evt_clone);
                    let _ = client.run().await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::ipc::get_snapshot,
            app::ipc::get_config,
            app::ipc::set_config,
            app::ipc::set_enabled,
            app::ipc::trash_user,
            app::ipc::restore_user,
            app::ipc::move_user,
            app::ipc::reset_counts,
            app::ipc::list_rewards,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
```

- [ ] **Step 2: Add `build.rs` at `src-tauri/build.rs` to expose env**

```rust
fn main() {
    if std::env::var("TWITCH_CLIENT_ID").is_err() {
        println!("cargo:rustc-env=TWITCH_CLIENT_ID=DEV_PLACEHOLDER_CLIENT_ID");
    }
    tauri_build::build()
}
```

- [ ] **Step 3: Compile**

```bash
cd src-tauri && cargo check
```
Address any error from missing `ipc.rs` symbols.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: wire tauri, http server, pipeline, eventsub in main"
```

---

## Task 14: Frontend — Zustand Store + IPC Wrapper

**Files:**
- Create: `src/ipc.ts`, `src/store.ts`

- [ ] **Step 1: Write `src/ipc.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";
import type { Config, Snapshot, Zone } from "./types";

export const ipc = {
  getSnapshot: () => invoke<Snapshot>("get_snapshot"),
  getConfig: () => invoke<Config>("get_config"),
  setConfig: (cfg: Config) => invoke<Snapshot>("set_config", { cfg }),
  setEnabled: (enabled: boolean) => invoke<Snapshot>("set_enabled", { enabled }),
  trashUser: (userId: string) => invoke<Snapshot>("trash_user", { userId }),
  restoreUser: (userId: string) => invoke<Snapshot>("restore_user", { userId }),
  moveUser: (userId: string, zone: Zone, index: number) =>
    invoke<Snapshot>("move_user", { userId, zone, index }),
  resetCounts: () => invoke<Snapshot>("reset_counts"),
  listRewards: () => invoke<Array<{ id: string; title: string; cost: number }>>("list_rewards"),
};
```

- [ ] **Step 2: Write `src/store.ts`**

```ts
import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import type { Config, Snapshot } from "./types";
import { ipc } from "./ipc";

type Store = {
  snap: Snapshot | null;
  config: Config | null;
  hydrate: () => Promise<void>;
};

export const useStore = create<Store>((set) => ({
  snap: null,
  config: null,
  hydrate: async () => {
    const [snap, config] = await Promise.all([ipc.getSnapshot(), ipc.getConfig()]);
    set({ snap, config });
  },
}));

listen<Snapshot>("state-changed", (e) => useStore.setState({ snap: e.payload }));
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(frontend): zustand store + ipc wrapper with event subscription"
```

---

## Task 15: Main Window UI (Header + Panes + Row)

**Files:**
- Create: `src/components/Header.tsx`, `src/components/Row.tsx`, `src/components/PlayingPane.tsx`, `src/components/WaitingPane.tsx`, `src/styles.css`
- Rewrite: `src/App.tsx`

- [ ] **Step 1: Write `src/components/Row.tsx`**

```tsx
import { useTranslation } from "react-i18next";
import type { User } from "../types";
import { ipc } from "../ipc";

const DAY = 24 * 60 * 60 * 1000;

export function Row({ user, zone }: { user: User; zone: "playing" | "waiting" }) {
  const { t } = useTranslation();
  const isFirst = user.lastJoinAt === null || (Date.now() - user.lastJoinAt) > DAY;
  return (
    <div className={`row ${zone}`} data-user-id={user.id} draggable>
      <span className="name">{user.displayName}</span>
      {isFirst && <span className="badge" title={t("row.badgeTitle")}>初</span>}
      <button className="trash-btn" title={t("row.sendToTrash")} onClick={() => ipc.trashUser(user.id)}>🗑</button>
    </div>
  );
}
```

- [ ] **Step 2: Write `src/components/PlayingPane.tsx`**

```tsx
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { Row } from "./Row";

export function PlayingPane() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  if (!snap || !config) return null;
  const slots = Array.from({ length: config.maxPlaying }, (_, i) => snap.playing[i]);
  return (
    <div className="pane" data-zone="playing">
      <h3>{t("panes.playing")} <span className="count">{snap.playing.length}/{config.maxPlaying}</span></h3>
      {slots.map((u, i) => u
        ? <Row key={u.id} user={u} zone="playing" />
        : <div key={`empty-${i}`} className="row empty">{t("panes.emptySlot")}</div>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Write `src/components/WaitingPane.tsx`**

```tsx
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { Row } from "./Row";

export function WaitingPane() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  if (!snap) return null;
  return (
    <div className="pane" data-zone="waiting">
      <h3>{t("panes.waiting")} <span className="count">{snap.waitingTotal} total</span></h3>
      {snap.waiting.length === 0
        ? <div className="empty">{t("panes.noWaiting")}</div>
        : snap.waiting.map((u) => <Row key={u.id} user={u} zone="waiting" />)
      }
    </div>
  );
}
```

- [ ] **Step 4: Write `src/components/Header.tsx`**

```tsx
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { ipc } from "../ipc";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

export function Header({ onOpenSettings }: { onOpenSettings: () => void }) {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  const enabled = snap?.enabled ?? false;
  const toggle = () => ipc.setEnabled(!enabled);
  const copyUrl = () => writeText(`http://localhost:${config?.port ?? 24816}/overlay`);

  return (
    <header className="app-header">
      <div className="brand">参加型 Manager</div>
      <button className={`toggle ${enabled ? "on" : "off"}`} onClick={toggle}>
        {enabled ? t("header.enabled") : t("header.disabled")}
      </button>
      <button onClick={copyUrl}>{t("header.copyUrl")}</button>
      <button onClick={onOpenSettings}>⚙ {t("header.settings")}</button>
    </header>
  );
}
```

- [ ] **Step 5: Rewrite `src/App.tsx`**

```tsx
import { useEffect, useState } from "react";
import { Header } from "./components/Header";
import { PlayingPane } from "./components/PlayingPane";
import { WaitingPane } from "./components/WaitingPane";
import { SettingsModal } from "./components/SettingsModal";
import { useStore } from "./store";
import "./styles.css";
import "./i18n";

export default function App() {
  const hydrate = useStore((s) => s.hydrate);
  const [settingsOpen, setSettingsOpen] = useState(false);
  useEffect(() => { hydrate(); }, [hydrate]);
  return (
    <div className="app">
      <Header onOpenSettings={() => setSettingsOpen(true)} />
      <main className="panes">
        <PlayingPane />
        <WaitingPane />
      </main>
      {settingsOpen && <SettingsModal onClose={() => setSettingsOpen(false)} />}
    </div>
  );
}
```

- [ ] **Step 6: Write `src/styles.css`**

```css
* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui; background: #1a1a22; color: #e8e8f0; }
.app { display: flex; flex-direction: column; height: 100vh; }
.app-header { display: flex; gap: 10px; padding: 10px 14px; border-bottom: 1px solid #2e2e3a; align-items: center; }
.brand { font-weight: 700; color: #8af; }
.toggle { margin-left: auto; padding: 4px 12px; border-radius: 14px; border: none; cursor: pointer; }
.toggle.on { background: #4c8; color: #000; }
.toggle.off { background: #555; color: #ccc; }
button { background: #2e2e3a; color: #e8e8f0; border: 1px solid #444; border-radius: 5px; padding: 4px 9px; cursor: pointer; }
button:hover { background: #3a3a46; }
.panes { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; padding: 12px; flex: 1; overflow: hidden; }
.pane { background: #14141b; border-radius: 8px; padding: 10px; overflow-y: auto; }
.pane h3 { margin: 0 0 8px 0; font-size: 13px; color: #8af; letter-spacing: 1px; text-transform: uppercase; }
.count { color: #666; font-size: 11px; font-weight: normal; }
.row { display: flex; align-items: center; gap: 8px; padding: 6px 10px; margin-bottom: 4px;
       background: #23232e; border-radius: 5px; font-size: 13px; cursor: grab; }
.row.playing { border-left: 3px solid #6af; }
.row.waiting { border-left: 3px solid #fc3; opacity: 0.9; }
.row.empty { opacity: 0.4; font-style: italic; cursor: default; }
.name { font-family: monospace; flex: 1; }
.badge { background: #fc3; color: #000; font-weight: 800; font-size: 10px; padding: 2px 6px; border-radius: 3px; }
.trash-btn { background: transparent; border: none; cursor: pointer; font-size: 14px; }
.empty { opacity: 0.5; text-align: center; padding: 20px; font-style: italic; }
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(ui): main window — header, panes, row"
```

---

## Task 16: Drag-Drop Within Main Window

**Files:**
- Modify: `src/components/Row.tsx`, `src/components/PlayingPane.tsx`, `src/components/WaitingPane.tsx`
- Create: `src/hooks/useDragDrop.ts`

- [ ] **Step 1: Create `src/hooks/useDragDrop.ts`**

```ts
import { useRef } from "react";
import type { Zone } from "../types";
import { ipc } from "../ipc";

export function useDragDrop() {
  const dragged = useRef<{ id: string } | null>(null);

  const onDragStart = (userId: string) => (e: React.DragEvent) => {
    dragged.current = { id: userId };
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", userId);
  };

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  };

  const onDrop = (zone: Zone, index: number) => (e: React.DragEvent) => {
    e.preventDefault();
    const id = dragged.current?.id ?? e.dataTransfer.getData("text/plain");
    if (!id) return;
    ipc.moveUser(id, zone, index);
    dragged.current = null;
  };

  return { onDragStart, onDragOver, onDrop };
}
```

- [ ] **Step 2: Update `Row.tsx` to accept drag handlers**

```tsx
import { useTranslation } from "react-i18next";
import type { User } from "../types";
import { ipc } from "../ipc";

const DAY = 24 * 60 * 60 * 1000;

export function Row({
  user, zone, index, onDragStart,
}: {
  user: User;
  zone: "playing" | "waiting";
  index: number;
  onDragStart: (userId: string) => (e: React.DragEvent) => void;
}) {
  const { t } = useTranslation();
  const isFirst = user.lastJoinAt === null || (Date.now() - user.lastJoinAt) > DAY;
  return (
    <div
      className={`row ${zone}`}
      draggable
      onDragStart={onDragStart(user.id)}
      data-index={index}
    >
      <span className="name">{user.displayName}</span>
      {isFirst && <span className="badge" title={t("row.badgeTitle")}>初</span>}
      <button className="trash-btn" title={t("row.sendToTrash")} onClick={() => ipc.trashUser(user.id)}>🗑</button>
    </div>
  );
}
```

- [ ] **Step 3: Update `PlayingPane.tsx` + `WaitingPane.tsx` to wire drag**

```tsx
// PlayingPane.tsx
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { Row } from "./Row";
import { useDragDrop } from "../hooks/useDragDrop";

export function PlayingPane() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  const { onDragStart, onDragOver, onDrop } = useDragDrop();
  if (!snap || !config) return null;
  const slots = Array.from({ length: config.maxPlaying }, (_, i) => snap.playing[i]);
  return (
    <div className="pane" data-zone="playing" onDragOver={onDragOver} onDrop={onDrop("playing", snap.playing.length)}>
      <h3>{t("panes.playing")} <span className="count">{snap.playing.length}/{config.maxPlaying}</span></h3>
      {slots.map((u, i) => u
        ? <div key={u.id} onDrop={onDrop("playing", i)} onDragOver={onDragOver}>
            <Row user={u} zone="playing" index={i} onDragStart={onDragStart} />
          </div>
        : <div key={`empty-${i}`} className="row empty" onDrop={onDrop("playing", i)} onDragOver={onDragOver}>
            {t("panes.emptySlot")}
          </div>
      )}
    </div>
  );
}
```

Apply analogous changes to `WaitingPane.tsx` (wire `onDragStart`, `onDragOver`, `onDrop("waiting", i)`).

- [ ] **Step 4: Manual smoke — run `bun run tauri dev`, drag a row between panes**

Expected: row moves, snapshot reflects new zone.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(ui): drag-drop between playing and waiting panes"
```

---

## Task 17: Settings Modal + Reset Confirm

**Files:**
- Create: `src/components/SettingsModal.tsx`

- [ ] **Step 1: Write `src/components/SettingsModal.tsx`**

```tsx
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { ipc } from "../ipc";
import type { Config, Language } from "../types";
import { ask } from "@tauri-apps/plugin-dialog";

export function SettingsModal({ onClose }: { onClose: () => void }) {
  const { t, i18n } = useTranslation();
  const current = useStore((s) => s.config);
  const [cfg, setCfg] = useState<Config | null>(current);
  const [rewards, setRewards] = useState<Array<{ id: string; title: string; cost: number }>>([]);

  useEffect(() => {
    ipc.listRewards().then(setRewards).catch(() => setRewards([]));
  }, []);

  if (!cfg) return null;

  const save = async () => {
    await ipc.setConfig(cfg);
    if (cfg.language) i18n.changeLanguage(cfg.language);
    onClose();
  };

  const reset = async () => {
    const confirmed = await ask(t("settings.resetWarning"), {
      title: t("settings.resetTitle"),
      kind: "warning",
      okLabel: t("settings.resetConfirm"),
      cancelLabel: t("settings.cancel"),
    });
    if (confirmed) {
      await ipc.resetCounts();
      onClose();
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>{t("settings.title")}</h2>
        <label>
          {t("settings.reward")}
          <select
            value={cfg.rewardId ?? ""}
            onChange={(e) => setCfg({ ...cfg, rewardId: e.target.value || null })}
          >
            <option value="">{t("settings.rewardAuto")}</option>
            {rewards.map((r) => <option key={r.id} value={r.id}>{r.title} ({r.cost})</option>)}
          </select>
        </label>
        <label>
          {t("settings.keyword")}
          <input value={cfg.keyword} onChange={(e) => setCfg({ ...cfg, keyword: e.target.value })} />
        </label>
        <label>
          {t("settings.maxPlaying")}
          <input type="number" min={1} max={10}
            value={cfg.maxPlaying}
            onChange={(e) => setCfg({ ...cfg, maxPlaying: Number(e.target.value) })} />
        </label>
        <label>
          {t("settings.maxWaiting")}
          <input type="number" min={1} max={20}
            value={cfg.maxWaiting}
            onChange={(e) => setCfg({ ...cfg, maxWaiting: Number(e.target.value) })} />
        </label>
        <label>
          <input type="checkbox" checked={cfg.autoPromote}
            onChange={(e) => setCfg({ ...cfg, autoPromote: e.target.checked })} />
          {t("settings.autoPromote")}
        </label>
        <label>
          {t("settings.language")}
          <select value={cfg.language} onChange={(e) => setCfg({ ...cfg, language: e.target.value as Language })}>
            <option value="ja">日本語</option>
            <option value="en">English</option>
            <option value="ko">한국어</option>
          </select>
        </label>
        <div className="modal-actions">
          <button onClick={reset} className="danger">{t("settings.reset")}</button>
          <span style={{ flex: 1 }} />
          <button onClick={onClose}>{t("settings.cancel")}</button>
          <button onClick={save} className="primary">{t("settings.save")}</button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Add modal styles to `src/styles.css`**

```css
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.modal { background: #1a1a22; padding: 20px; border-radius: 8px; min-width: 400px; display: flex; flex-direction: column; gap: 10px; }
.modal h2 { margin: 0 0 10px 0; }
.modal label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; }
.modal input, .modal select { background: #0e0e14; color: #e8e8f0; border: 1px solid #444; padding: 4px 6px; border-radius: 3px; }
.modal-actions { display: flex; gap: 8px; margin-top: 10px; }
button.danger { background: #c44; color: #fff; }
button.primary { background: #4c8; color: #000; }
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(ui): settings modal with reset confirm dialog"
```

---

## Task 18: Trash Window

**Files:**
- Create: `src/trash-entry.tsx`, `src/Trash.tsx`
- (`trash.html` already created in Task 1)

- [ ] **Step 1: Write `src/trash-entry.tsx`**

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import { Trash } from "./Trash";
import "./styles.css";
import "./i18n";

ReactDOM.createRoot(document.getElementById("root")!).render(<Trash />);
```

- [ ] **Step 2: Write `src/Trash.tsx`**

```tsx
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "./store";
import { ipc } from "./ipc";

export function Trash() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const hydrate = useStore((s) => s.hydrate);
  useEffect(() => { hydrate(); }, [hydrate]);
  if (!snap) return null;
  return (
    <div className="app">
      <header className="app-header"><div className="brand">{t("panes.trash")}</div></header>
      <main style={{ padding: 12, overflowY: "auto", flex: 1 }}>
        {snap.trash.length === 0
          ? <div className="empty">—</div>
          : snap.trash.map((u) => (
              <div key={u.id} className="row" style={{ opacity: 0.6 }}>
                <span className="name" style={{ textDecoration: "line-through" }}>{u.displayName}</span>
                <button onClick={() => ipc.restoreUser(u.id)}>↩ {t("row.restore")}</button>
              </div>
            ))}
      </main>
    </div>
  );
}
```

- [ ] **Step 3: Add header-button in main window to show trash window**

Edit `src/components/Header.tsx`, add handler:

```tsx
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
const openTrash = async () => {
  const existing = await WebviewWindow.getByLabel("trash");
  if (existing) { await existing.show(); await existing.setFocus(); }
};
```

Add button `<button onClick={openTrash}>🗑 {t("panes.trash")}</button>` between copy-URL and settings.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(ui): separate trash window with restore buttons"
```

---

## Task 19: i18n Wiring (Frontend)

**Files:**
- Create: `src/i18n.ts`

- [ ] **Step 1: Write `src/i18n.ts`**

```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ja from "../locales/ja.json";
import en from "../locales/en.json";
import ko from "../locales/ko.json";

i18n.use(initReactI18next).init({
  resources: { ja: { translation: ja }, en: { translation: en }, ko: { translation: ko } },
  lng: (navigator.language.startsWith("ko") ? "ko" : navigator.language.startsWith("en") ? "en" : "ja"),
  fallbackLng: "ja",
  interpolation: { escapeValue: false, prefix: "{", suffix: "}" },
});

export default i18n;
```

- [ ] **Step 2: Enable Vite JSON import**

Ensure `tsconfig.json` has:
```json
{ "compilerOptions": { "resolveJsonModule": true, "esModuleInterop": true } }
```

- [ ] **Step 3: Sync language from config on hydrate**

Edit `src/store.ts` `hydrate`:

```ts
hydrate: async () => {
  const [snap, config] = await Promise.all([ipc.getSnapshot(), ipc.getConfig()]);
  set({ snap, config });
  (await import("./i18n")).default.changeLanguage(config.language);
},
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(i18n): wire i18next with ja/en/ko + sync from config"
```

---

## Task 20: Auto-Update via GitHub Releases

**Files:**
- Modify: `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src/App.tsx`

- [ ] **Step 1: Add updater config to `src-tauri/tauri.conf.json`**

```json
"plugins": {
  "updater": {
    "active": true,
    "endpoints": [
      "https://github.com/YOUR_GH_USER/twitch-sankagata-manager/releases/latest/download/latest.json"
    ],
    "dialog": false,
    "pubkey": "REPLACE_WITH_PUB_KEY_GENERATED_BY_TAURI_CLI"
  }
}
```

Generate key pair:
```bash
bun run tauri signer generate -w ~/.tauri/twitch-sankagata.key
```

Store **private key path** in CI secrets as `TAURI_SIGNING_PRIVATE_KEY`. Paste **public key** into config.

- [ ] **Step 2: Add update check logic (frontend)**

Create `src/update.ts`:

```ts
import { check } from "@tauri-apps/plugin-updater";
import { ask } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import i18n from "./i18n";

export async function checkForUpdate() {
  try {
    const update = await check();
    if (update?.available) {
      const ok = await ask(i18n.t("errors.updateReady"), { okLabel: "OK", cancelLabel: "Later" });
      if (ok) {
        await update.downloadAndInstall();
        await relaunch();
      }
    }
  } catch (e) { console.error("update check failed", e); }
}
```

Call from `App.tsx` on mount:

```tsx
useEffect(() => { import("./update").then(m => m.checkForUpdate()); }, []);
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(update): tauri-plugin-updater with github releases"
```

---

## Task 21: GitHub Actions Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write `.github/workflows/release.yml`**

```yaml
name: release
on:
  push:
    tags: ["v*"]

jobs:
  build:
    permissions: { contents: write }
    strategy:
      matrix:
        os: [windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v2
      - uses: dtolnay/rust-toolchain@stable
      - name: install frontend deps
        run: bun install
      - name: build + release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          TWITCH_CLIENT_ID: ${{ secrets.TWITCH_CLIENT_ID }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Twitch Sankagata Manager ${{ github.ref_name }}"
          releaseDraft: true
          prerelease: false
```

- [ ] **Step 2: Document secrets in `README.md`**

(See Task 23 for README.)

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "ci: release workflow for tagged versions with tauri-action"
```

---

## Task 22: Frontend Unit Tests

**Files:**
- Create: `src/components/__tests__/Row.test.tsx`, `vitest.config.ts`

- [ ] **Step 1: Write `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: { environment: "jsdom", setupFiles: ["./src/test-setup.ts"] },
});
```

Create `src/test-setup.ts`:
```ts
import "@testing-library/jest-dom";
// mock Tauri invoke so components can import ipc
(globalThis as any).__TAURI_INTERNALS__ = {
  invoke: async () => null,
  transformCallback: () => 0,
};
```

- [ ] **Step 2: Write `src/components/__tests__/Row.test.tsx`**

```tsx
import { render, screen } from "@testing-library/react";
import { Row } from "../Row";
import { describe, it, expect } from "vitest";
import "../../i18n";

const user = {
  id: "u1", name: "alice", displayName: "Alice",
  joinCount: 0, lastJoinAt: null, enqueuedAt: 0, manualOrder: null,
};

describe("Row", () => {
  it("shows badge when first time today (lastJoinAt null)", () => {
    render(<Row user={user} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.getByText("初")).toBeInTheDocument();
  });

  it("hides badge when user joined within 24h", () => {
    const recent = { ...user, lastJoinAt: Date.now() - 60_000 };
    render(<Row user={recent} zone="playing" index={0} onDragStart={() => () => {}} />);
    expect(screen.queryByText("初")).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 3: Run**

```bash
bun run vitest run
```
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(ui): row badge rendering"
```

---

## Task 23: README + Manual Acceptance Checklist

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write `README.md`**

```markdown
# Twitch Sankagata Manager

Twitch 参加型 queue manager with OBS overlay.

## Dev

```bash
bun install
bun run tauri dev
```

Set `TWITCH_CLIENT_ID` env var to your registered Twitch application's client ID.

## Build

```bash
bun run tauri build
```

## Release

Create a tag `v*`, push — GitHub Actions builds Windows + macOS installers with auto-updater manifest.

Required secrets:
- `TAURI_SIGNING_PRIVATE_KEY` — tauri updater signing key
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — (if key was password-protected)
- `TWITCH_CLIENT_ID` — public Twitch application ID
```

- [ ] **Step 2: Final manual acceptance run-through**

Execute each checkbox against a running debug build (`bun run tauri dev`):

- [ ] First-run auth flow opens browser with user code
- [ ] OBS Browser Source at `http://localhost:24816/overlay` shows transparent bg
- [ ] Redeeming channel-point item on Twitch adds row within 2 s
- [ ] Refunding redemption removes row
- [ ] Drag between Playing/Waiting reorders correctly
- [ ] Trash window opens via header button, shows removed users, restore works
- [ ] Settings modal saves; language switch (ja/en/ko) updates all strings
- [ ] Reset counts prompts confirm dialog; proceeds only on OK
- [ ] Kill network for 30 s — status dot turns yellow, auto-reconnects
- [ ] Second app launch focuses existing window instead of opening new one
- [ ] Closing + reopening app preserves queue, trash, and config

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "docs: readme and manual acceptance checklist"
```

---

## Self-Review Summary

Coverage check against spec sections:
- Goals → tasks 4, 5, 15, 16, 17, 19, 20
- Tech Stack → tasks 1, 2
- Architecture → task 13 (main wiring)
- Components (Rust) → tasks 3-13
- Components (Frontend) → tasks 14-19
- Components (Overlay) → task 10
- Data Flow (auth) → task 7
- Data Flow (EventSub + redemption add) → tasks 8, 9, 11
- Data Flow (refund) → task 11 (pipeline update branch)
- Data Flow (manual ops) → task 12 (IPC) + 16 (drag) + 17 (reset)
- Data Flow (auto-promote) → task 5
- Data Flow (priority ordering) → task 4
- Data Flow (reset) → task 5 + task 17 (confirm dialog)
- Data Flow (startup) → task 13
- EventSub reconnect → task 8
- i18n → tasks 2, 19
- Error handling (port fallback, keyring, refresh) → tasks 6, 7, 10 (bind_with_fallback)
- Auto-update → tasks 20, 21
- Testing → tasks 3-10, 22
- Single-instance → task 13

**Gaps noted:**
- Full connection-status indicator UI (🟢/🟡/🔴 in header) is not explicitly tasked — add in a follow-up if visual indicator needed beyond what toggle already shows.
- macOS signing deferred (spec explicitly deferred).
- Helix 429 rate-limit retry logic not wired (simple retry loop; can be added to helix.rs if observed).

These gaps are either explicitly deferred (spec non-goals / open questions) or minor enhancements — none block v1.
