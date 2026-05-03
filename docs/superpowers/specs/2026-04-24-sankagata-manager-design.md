# 参加型 Manager — Design Spec

**Date:** 2026-04-24
**Status:** Draft, pending user review

## Purpose

Desktop app for Japanese-culture Twitch streamers who let viewers join team-based games (LoL, Valorant, etc.) via channel point redemptions. App detects redemptions, queues viewers, renders an OBS overlay, and gives the streamer a management UI to manually coordinate who plays next.

## Goals

- Zero CLI exposure — beginner-friendly double-click `.exe`
- Real-time OBS overlay (transparent background, ~35% alpha rows)
- Manual queue control (drag-drop within panes, send-to-trash, restore)
- Automatic promotion from waiting → playing when a slot frees
- First-time-today priority with 初 badge, rolling 24h
- Refund detection (auto-remove if streamer cancels a redemption)
- Multi-language UI: ja / en / ko
- Auto-update via GitHub Releases

## Non-Goals (first version)

- Cloud sync / multi-machine
- Chat commands or viewer-facing features
- Custom badge upload
- Per-game / per-stream separate queues
- Stats export / leaderboards
- Mobile app

## Tech Stack

| Layer | Choice | Rationale |
|---|---|---|
| App shell | Tauri 2.x | Small binary (~10 MB), multi-window, mature updater |
| Backend language | Rust | Strong EventSub/axum crates, OS keychain via `keyring` |
| Frontend | React + Vite + TypeScript | Familiar, fast iteration, good Tauri tooling |
| HTTP server | `axum` + `tokio` | Async, ergonomic, standard choice |
| EventSub client | `tokio-tungstenite` | Async WebSocket |
| Helix client | `reqwest` | Standard async HTTP |
| Persistence | `rusqlite` (bundled) | Single file DB, no external deps |
| Secret storage | `keyring` crate | OS-native (Windows Credential Manager / macOS Keychain) |
| i18n | `i18next` (frontend) + `rust-i18n` (backend) | Shared JSON locale files |
| State | Zustand (frontend), `tokio::sync::Mutex<AppState>` (backend) | Simple, minimal |
| Auto-update | `tauri-plugin-updater` | Reads GitHub Releases feed |

**Assumption:** developer is comfortable writing Rust for the backend. Tauri + Node sidecar remains a fallback option if Rust work becomes blocking during implementation.

## Architecture

```
┌────────────────────────────────────────────────────────┐
│  Tauri app process (single .exe)                       │
│                                                        │
│  ┌──────────────┐      ┌─────────────────────────┐    │
│  │ Rust backend │◄────►│ State (SQLite + memory) │    │
│  │   (tokio)    │      └─────────────────────────┘    │
│  │              │                                      │
│  │ • axum HTTP server @ localhost:24816                │
│  │     GET  /overlay     (HTML for OBS)                │
│  │     GET  /ws          (WebSocket, push to overlay)  │
│  │     GET  /auth/callback                             │
│  │                                                     │
│  │ • EventSub WebSocket client (→ Twitch)              │
│  │ • OAuth device-code flow manager                    │
│  │ • keyring token store (OS keychain)                 │
│  └──────────────┘                                      │
│         ▲                                              │
│         │ Tauri IPC (invoke / emit)                    │
│         ▼                                              │
│  ┌──────────────────────┐   ┌──────────────────┐       │
│  │ Main window (Webview)│   │ Trash window     │       │
│  │ Playing | Waiting    │   │ (separate)       │       │
│  │ drag within window   │   │ restore button   │       │
│  └──────────────────────┘   └──────────────────┘       │
└────────────────────────────────────────────────────────┘
               │
               ▼ (browser source URL)
         ┌──────────────────────────────────┐
         │ OBS Studio — Browser Source      │
         │ http://localhost:24816/overlay   │
         └──────────────────────────────────┘
```

### Key points

- Single `.exe`; streamer never sees a terminal
- Rust backend owns all state and Twitch connections
- Two Tauri webview windows = management UI
- Overlay is **not** a Tauri window — it's HTML served on localhost so OBS Browser Source can load it
- Local WebSocket pushes state to the overlay in real time (sub-100 ms updates)
- EventSub WebSocket from Rust → Twitch receives redemption add and update (refund) events

## Components

### Rust backend (`src-tauri/src/`)

| Module | Purpose |
|---|---|
| `main.rs` | Tauri app init, window spawn, invoke handler registration |
| `server.rs` | axum HTTP server — `/overlay`, `/ws`, `/auth/callback` |
| `eventsub.rs` | Twitch EventSub WebSocket client, reconnect, subscribe to redemption add + update |
| `auth.rs` | OAuth device-code flow, token refresh, keyring storage |
| `helix.rs` | Helix API client — list rewards, refund redemption (`PATCH /channel_points/custom_rewards/redemptions`) |
| `state.rs` | In-memory queue state, mutex-guarded, emits change events |
| `store.rs` | SQLite persistence — users, counts, trash, config, manual order |
| `priority.rs` | First-time-today logic, rolling 24h eval, queue ordering |
| `ipc.rs` | Tauri invoke command handlers (frontend → backend) |
| `i18n.rs` | Locale JSON loader, `t(key)` helper |
| `single_instance.rs` | Enforced via `tauri-plugin-single-instance`; second launch focuses existing window |

### Frontend (`src/` — React + Vite + TypeScript)

| File | Purpose |
|---|---|
| `App.tsx` | Main window: `<Header/>` + `<PlayingPane/>` + `<WaitingPane/>` |
| `TrashWindow.tsx` | Separate Tauri window — trash list with restore buttons |
| `SettingsModal.tsx` | Reward picker, keyword, max-playing, max-waiting, auto-promote, language, reset |
| `Header.tsx` | Twitch auth status, ON/OFF toggle, copy-OBS-URL, gear icon |
| `Row.tsx` | Single user row — drag handle, name, badge, trash button |
| `useStore.ts` | Zustand store subscribed to backend emits |
| `ipc.ts` | Typed wrapper around `invoke` |
| `i18n/` | i18next config + `locales/{ja,en,ko}.json` |

### Overlay (`overlay/` — bundled into Rust binary via `include_str!`)

| File | Purpose |
|---|---|
| `overlay.html` | Minimal skeleton, loads JS |
| `overlay.css` | Per-row translucent pills (~35% alpha), text-shadow for legibility |
| `overlay.js` | Connects to `ws://localhost:PORT/ws`, renders snapshot, simple fade animation on add/remove |

### Shared types

```ts
type User = {
  id: string;              // Twitch user id
  name: string;            // login name
  displayName: string;     // display_name (supports JP/KR chars)
  joinCount: number;
  lastJoinAt: number | null;  // unix ms UTC
  enqueuedAt: number;
  manualOrder: number | null;  // set when streamer drags
};

type Zone = "playing" | "waiting" | "trash";

type Config = {
  rewardId: string | null;   // null = keyword fallback
  keyword: string;           // default "参加"
  maxPlaying: number;        // default 4
  maxWaiting: number;        // default 3 (overlay visible count)
  autoPromote: boolean;      // default true
  enabled: boolean;
  language: "ja" | "en" | "ko";
  port: number;              // default 24816
};

type Snapshot = {
  type: "state";
  playing: User[];
  waiting: User[];       // includes overflow beyond maxWaiting
  waitingTotal: number;  // = waiting.length
  trash: User[];
  enabled: boolean;
  language: string;
};
```

## Data Flow

### First-run auth (OAuth Device Code Flow)

Avoids shipping a client secret in the binary. A public Twitch application (registered once by the project maintainer) provides the `client_id`; the device-code flow never needs a client secret, so the ID can be embedded in the binary safely.

```
Streamer clicks "Login with Twitch"
  → backend: POST https://id.twitch.tv/oauth2/device
      scopes: channel:read:redemptions, channel:manage:redemptions
  → response: { device_code, user_code, verification_uri }
  → Tauri opens verification_uri in default browser
  → UI shows user_code ("Enter this: ABCD-1234")
  → backend polls POST /oauth2/token with device_code every 5s
  → receives { access_token, refresh_token }
  → keyring.set("sankagata-twitch", tokens)
  → backend GETs /users → stores broadcaster_id
  → subscribes EventSub
```

### EventSub subscription + redemption add

```
Startup (or post-auth):
  1. Connect ws://eventsub.wss.twitch.tv/ws
  2. Receive session_welcome → capture session_id
  3. POST /helix/eventsub/subscriptions, transport=websocket, session_id
       - channel.channel_points_custom_reward_redemption.add
       - channel.channel_points_custom_reward_redemption.update

On .add event:
  event → eventsub.rs filter:
    if config.rewardId AND event.reward.id != config.rewardId → skip
    elif !config.rewardId AND !event.reward.title.contains(config.keyword) → skip
  → state.add_redemption(user) (dedupe if already in playing/waiting)
  → store.persist()
  → tauri::emit("state-changed", snapshot)   // all Tauri windows
  → overlay_ws.broadcast(snapshot)           // OBS Browser Source
  → if auto_promote conditions met → promote
```

### Refund handling (.update event with status = CANCELED)

```
→ state.on_refund(user_id, reward_id)
    - if user in waiting → remove
    - if user in playing → remove; if auto_promote → promote next waiting
    - if user in trash → leave
    - decrement joinCount by 1 (may restore "first-time" status)
→ emit / broadcast
```

### Manual operations (frontend → backend via `invoke`)

| Command | Payload | Effect |
|---|---|---|
| `move_user` | `{ userId, zone, index }` | Drag within main window; sets `manualOrder` |
| `trash_user` | `{ userId }` | Send to trash |
| `restore_user` | `{ userId }` | From trash back to end of waiting |
| `set_enabled` | `{ enabled }` | ON/OFF toggle; EventSub stays connected, just filters out adds |
| `set_config` | `{ config }` | Save settings |
| `reset_counts` | — | After confirmation dialog, `UPDATE users SET joinCount=0, lastJoinAt=NULL` |
| `list_rewards` | — | Calls Helix, returns array for dropdown |
| `re_auth` | — | Starts device-code flow again |

Every invoke → state mutation → persist → emit to windows + overlay WS.

### Auto-promote

Triggered after: new redemption, trash, drag out of playing, config change, refund of a playing user.

```
while playing.len < maxPlaying AND waiting.len > 0 AND config.autoPromote:
  promoted = waiting.shift()
  promoted.joinCount += 1
  promoted.lastJoinAt = now_utc_ms()
  playing.push(promoted)
```

### Priority ordering (waiting list)

```
Sort waiting by:
  1. manualOrder IS NOT NULL — respect streamer override first
  2. isFirstTimeToday (lastJoinAt NULL OR (now - lastJoinAt) > 24h) DESC
  3. enqueuedAt ASC  (FIFO within same tier)
```

Badge `初` renders when `lastJoinAt` is null OR `(now - lastJoinAt) > 24h`.

### Reset button

```
Click "Reset counts" → tauri-plugin-dialog ask():
  title:   t("settings.resetTitle")
  message: t("settings.resetWarning")
  buttons: [Cancel, Reset]

if confirmed:
  UPDATE users SET joinCount=0, lastJoinAt=NULL, manualOrder=NULL;
  emit snapshot  (all badges return to 初)
```

Trash and current queue contents are preserved — only counts/history wipe.

### App startup

```
1. Create SQLite DB if missing, run migrations
2. Load config, users, playing, waiting, trash from DB
3. Start axum on config.port; fallback to next free port (try +1..+10); update UI with actual port
4. If token in keyring → validate with /users; else show login screen
5. If token expired → refresh; if refresh fails → clear + prompt re-login
6. Enforce single-instance (`tauri-plugin-single-instance`); second launch focuses existing window and exits
7. Spawn main window; trash window hidden by default
8. Subscribe EventSub
```

### EventSub reconnect

- On WS close: exponential backoff 1s → 2s → 4s → ... capped at 60s
- On reconnect: new `session_id`, resubscribe
- Honor Twitch `session_reconnect` message — connect to provided URL, migrate seamlessly
- Connection status dot in header: 🟢 connected / 🟡 reconnecting / 🔴 auth error

## i18n

Languages: **ja** (default for JP streamers), **en**, **ko**.

- Frontend library: `i18next` + `react-i18next`
- Backend library: `rust-i18n`
- Shared JSON locale files under `locales/{ja,en,ko}.json` (read by both sides)
- On first run, detect default via `navigator.language`; fallback `ja`
- Stored in `config.language`, synced to backend via IPC on change
- Overlay strings localized too (`+N more waiting`, etc.)
- Badge text `初` stays universal across locales (it's a cultural signal, not a UI string)
- OBS URL string (`http://localhost:...`) never translated

Example keys (`ja.json`):

```json
{
  "header": { "login": "Twitchでログイン", "copyUrl": "OBS URLをコピー" },
  "panes": { "playing": "▶ プレイ中", "waiting": "⏳ 待機中", "trash": "🗑 ゴミ箱" },
  "settings": {
    "reward": "対象リワード",
    "keyword": "キーワード",
    "maxPlaying": "最大プレイ人数",
    "autoPromote": "自動昇格",
    "reset": "カウントをリセット",
    "resetTitle": "リセットの確認",
    "resetWarning": "すべての参加回数をリセットします。元に戻せません。"
  },
  "overlay": { "moreWaiting": "+ {n} 人待機" },
  "errors": {
    "noAffiliate": "チャンネルポイントを使用するにはアフィリエイト以上が必要です",
    "portInUse": "ポート{port}が使用中。他のアプリを閉じてください",
    "reconnecting": "再接続中..."
  }
}
```

## Error Handling

### Auth
- No token on startup → show login screen; EventSub not started
- 401 on Helix → auto-refresh access token; retry once
- Refresh token invalid → clear keyring, prompt re-login, disable ON toggle
- Device-code poll timeout (15 min) → "Code expired, try again"
- Network down during auth → exponential backoff retry + "offline, retrying..." toast
- Scope mismatch → detect on `/users`, force re-auth

### EventSub
- WS close (1006, 4xxx) → exponential backoff reconnect (1 → 60 s), yellow status
- `session_reconnect` → connect to new URL, migrate
- Subscription revoked (rewards changed) → re-list, reconcile, toast if stale
- Missed 10-min keepalive → force reconnect

### HTTP server
- Port 24816 in use → try +1..+10; if all fail, show fatal dialog
- Overlay WS client disconnect (OBS reload) → drop silently
- Malformed request → 400, log

### Database
- File locked → retry with sleep, 3 attempts
- Corruption on open → rename to `state.db.corrupt-{ts}`, start fresh, toast "state reset"
- Migration failure → refuse start, dialog with file path
- Disk full → toast, keep running from memory, retry periodically

### State consistency
- All mutations behind single `tokio::sync::Mutex<AppState>` — no race between EventSub and IPC
- User can exist in **one zone only**; transitions atomic inside lock
- Duplicate redemption for same user → dedupe by `user_id`, log
- Trash cap = 200; oldest FIFO evicted beyond cap

### Helix API
- 429 → respect `Ratelimit-Reset`, queue
- 404 reward ID → fall back to keyword, toast
- 500/503 → retry 3× with backoff, log
- Non-affiliate broadcaster → detected on login, warn (channel points require affiliate)

### UI error surfaces
- Toast system for non-fatal (bottom-right, 5s auto-dismiss)
- Status dot in header with tooltip
- Fatal → `tauri-plugin-dialog` modal
- Logs: `%APPDATA%/twitch-sankagata-manager/logs/YYYY-MM-DD.log`, rotated 7 days; "Open Log Folder" button in settings

### Panics
- Rust panic handler → log with stack → dialog with log path → clean exit

## Auto-Update

Standard `tauri-plugin-updater` + GitHub Releases.

1. Developer pushes tag `v1.2.3`
2. GitHub Actions runs `tauri-action` → builds Windows NSIS installer + macOS `.dmg` + `latest.json` manifest → uploads to Release
3. On app launch (and every 4 hours while running), client polls `latest.json`
4. If new version found → downloads in background → prompts "Update ready, restart to install?"
5. On restart, installer runs silently, relaunches app

**Code signing:** ship unsigned initially (Windows SmartScreen warns "Unknown publisher" first install). Evaluate Azure Trusted Signing (~$10/mo) if audience grows.

**User data** (SQLite file, keyring token) lives in `app_data_dir()` — untouched across updates.

## Testing

### Rust unit (`cargo test`)
| Module | Key tests |
|---|---|
| `priority.rs` | first-time logic at null / 23h59m / 24h1m; sort stability; manualOrder override |
| `state.rs` | add / remove / trash / restore transitions; dedup; auto-promote; zone exclusivity |
| `auth.rs` | token refresh; scope check; keyring mock |
| `eventsub.rs` | event filter (reward ID, keyword); refund decrement |
| `store.rs` | schema migration; round-trip persistence; corruption recovery |

### Integration (`tests/`)
- Spawn axum on random port, inject synthetic EventSub events, assert overlay WS receives correct snapshot
- Mock Twitch with `wiremock` for auth + Helix flows

### Frontend (Vitest + React Testing Library)
- `Row.test.tsx` — renders 初 badge when first-time; trash button invokes IPC
- `useStore.test.ts` — state sync from emit
- `SettingsModal.test.tsx` — reset opens confirm dialog
- Mock Tauri `invoke` via `__TAURI_INTERNALS__` shim

### E2E
- Tauri E2E tooling is immature as of 2026-04 — defer full Playwright coverage
- Overlay smoke: load `http://localhost:PORT/overlay` in headless Chromium, assert rows match state snapshot

### Manual acceptance checklist (pre-release)
- [ ] First-run auth completes on Windows + macOS
- [ ] OBS Browser Source loads overlay, transparent bg works
- [ ] Redeem on Twitch → row appears within 2 s
- [ ] Refund redemption on Twitch dashboard → row disappears
- [ ] Drag between Playing/Waiting persists after restart
- [ ] Trash window opens, restore works, survives restart
- [ ] Reset button shows confirm, wipes counts, irreversible
- [ ] Language switch (ja/en/ko) updates all strings incl. overlay
- [ ] Auto-update prompts on new GitHub Release
- [ ] EventSub reconnects after 30 s network blip
- [ ] Second instance refuses to run (port conflict handled)

## Directory Layout

```
twitch-sankagata-manager/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── server.rs
│   │   ├── eventsub.rs
│   │   ├── auth.rs
│   │   ├── helix.rs
│   │   ├── state.rs
│   │   ├── store.rs
│   │   ├── priority.rs
│   │   ├── ipc.rs
│   │   └── i18n.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── TrashApp.tsx
│   ├── components/
│   │   ├── Header.tsx
│   │   ├── Row.tsx
│   │   ├── PlayingPane.tsx
│   │   ├── WaitingPane.tsx
│   │   └── SettingsModal.tsx
│   ├── store/useStore.ts
│   ├── ipc.ts
│   └── i18n/
├── overlay/                # Static HTML bundled into binary
│   ├── overlay.html
│   ├── overlay.css
│   └── overlay.js
├── locales/                # Shared ja/en/ko JSON
│   ├── ja.json
│   ├── en.json
│   └── ko.json
├── docs/
│   └── superpowers/specs/
├── .github/workflows/release.yml
├── package.json
├── vite.config.ts
└── SPEC.md
```

## Open Questions (to resolve in planning)

- Sort tiebreaker when two users both have `manualOrder` set? → Use `manualOrder` value ascending; on drag, rewrite ordinals to avoid collisions.
- Does "disabled" (`config.enabled = false`) also pause the overlay WS, or just skip adds? → Skip adds only; existing list stays visible.
- Should first-time priority consider trash (i.e. does going to trash count as "joined")? → No. Trash only reflects streamer action; `joinCount` only increments on `waiting → playing` transition.
- Cross-platform note: macOS signing differs from Windows; defer macOS signing to a later milestone.

---

*End of spec.*
