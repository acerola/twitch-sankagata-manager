<p align="center">
  <img src="site/assets/icon-256.png" width="128" height="128" alt="Twitch Sankagata Manager">
</p>

<h1 align="center">Twitch Sankagata Manager</h1>

<p align="center">
  Twitch 参加型 viewer-participation queue manager with OBS overlay.<br>
  Detects channel-point redemptions, manages a playing/waiting/trash queue,<br>
  and serves a transparent OBS Browser Source overlay in real time.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Twitch-EventSub-9146FF?style=flat-square&logo=twitch" alt="Twitch EventSub">
  <img src="https://img.shields.io/badge/OBS-Overlay-ffffff?style=flat-square&logo=obs-studio" alt="OBS Overlay">
  <img src="https://img.shields.io/badge/Platforms-Windows%20|%20macOS-cccccc?style=flat-square" alt="Platforms">
  <img src="https://img.shields.io/github/v/release/acerola/twitch-sankagata-manager?style=flat-square" alt="Release">
</p>

---

## Features

| Capability | Detail |
|---|---|
| **Real-time pickup** | Channel-point redemption via Twitch EventSub WebSocket |
| **Queue zones** | Playing / Waiting / Trash with drag-and-drop reordering |
| **First-timer badges** | First-time-today markers + configurable prioritisation |
| **OBS overlay** | Built-in HTTP server + native transparent overlay window |
| **Themes** | Seven themes — Twitch, Midnight, Daylight, Sakura, Forest, High Contrast, Custom |
| **Languages** | 日本語 / English / 한국어 |
| **Mock mode** | Local testing with Twitch CLI, no real stream required |
| **Auto-updater** | GitHub Releases + Tauri updater |

---

## Install

Pre-built installers are on the **[Releases page](https://github.com/acerola/twitch-sankagata-manager/releases)**.

---

## Quick Start

```bash
bun install
cp .env.example .env        # set TWITCH_CLIENT_ID
bun run tauri dev
```

> Register your Twitch app at **[dev.twitch.tv/console/apps](https://dev.twitch.tv/console/apps)** — Client Type: **Public**, Redirect: `http://localhost` (placeholder; unused). No client secret needed (device-code flow).

`src-tauri/build.rs` auto-loads `.env`; shell-exported vars override it. After changing values, run `cd src-tauri && cargo clean` once if the compile cache is stale.

---

## Testing

```bash
bun run test                    # frontend (vitest)
cd src-tauri && cargo test     # backend (rust)
```

---

## Build

```bash
bun run tauri build
```

---

## Mock Mode — Local Testing with Twitch CLI

Test channel-point redemption events locally without a real Twitch stream.

### Setup

**1. Install Twitch CLI**

```bash
# Windows (Scoop)
scoop bucket add twitch https://github.com/twitchdev/scoop-bucket.git
scoop install twitch-cli

# macOS / Linux (Homebrew)
brew install twitchdev/twitch/twitch-cli
```

**2. Start the mock WebSocket server**

```bash
twitch event websocket start-server --port=8081
```

**3. Enable mock mode in the app**

Header → **Debug** → toggle **Mock Mode** at the top of the Twitch CLI Mock section.

**4. Trigger mock events**

Copy the prepared command from the Debug panel, paste in the terminal, replace `REWARD_ID` with your actual reward ID.

### Useful commands

```bash
# Single redemption
twitch event trigger add-redemption \
  -F ws://localhost:8081 \
  -t BROADCASTER_ID -f USER_ID -i REWARD_ID -S fulfilled

# Burst of 5
twitch event trigger add-redemption \
  -F ws://localhost:8081 \
  -t BROADCASTER_ID -f USER_ID -i REWARD_ID -S fulfilled -c 5
```

### How it works

- Mock mode switches EventSub WebSocket from `wss://eventsub.wss.twitch.tv/ws` to `ws://localhost:8081`
- The mock server forwards events exactly like real Twitch
- Disabling mock mode reconnects to production automatically
- `bun run tauri dev` shows verbose logging on stderr

