# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.5] - 2026-05-03

### Changed
- Renamed the app, package, bundle, release assets, site copy, and repository links to Twitch Sankagata Manager / `twitch-sankagata-manager`.
- Renamed the default theme id to `twitch` while accepting the old `mumamuma` value for existing saved config.
- Changed the Tauri app identifier and keyring service to the new name, with one-shot migration from the previous state database and Twitch token entry.

## [0.2.4] - 2026-05-03

### Fixed
- Native overlay window hover now works across the transparent empty area, so the drag grip appears reliably in packaged builds.
- Overlay auto-resize now skips duplicate same-height native `setSize` calls to avoid resize churn.

### Changed
- Startup does less work before first paint by sharing the initial auth status with the header and delaying the update check briefly.
- Queue persistence now writes visible queue state in one SQLite transaction and releases the app-state lock before disk I/O.

## [0.1.10] - 2026-05-02

### Fixed
- `latest.json` macOS URL pointed to `Twitch.Sankagata.Manager_universal.app.tar.gz` but Tauri's universal-darwin build outputs `Twitch.Sankagata.Manager.app.tar.gz` (no `_universal_` infix). Mac auto-update 404'd. URL updated to match the actual asset name.

## [0.1.9] - 2026-05-02

### Fixed
- `latest.json` builder pointed at `*_x64-setup.nsis.zip` and looked for `*_x64-setup.nsis.zip.sig`. Tauri v2 with `createUpdaterArtifacts: true` signs the NSIS `.exe` installer directly on Windows (no `.nsis.zip` wrapper), so both inputs were empty in the manifest job. Switch to `*_x64-setup.exe(.sig)`.

## [0.1.8] - 2026-05-02

### Fixed
- Release workflow's `collect outputs` step now uses `find` instead of `shopt -s globstar` shell glob expansion. macOS runners still ship bash 3.2 which rejects `globstar` as an invalid shell option name, breaking the macOS half of the build. `find` works on both Git Bash 4+ and Apple bash 3.2.

## [0.1.7] - 2026-05-02

### Fixed
- Release workflow strips whitespace and line breaks from `TAURI_SIGNING_PRIVATE_KEY` before invoking `tauri build`. The secret was line-wrapped base64 (76-column wrap from `base64 < key`); native tauri-cli's decoder choked on embedded newlines with `Invalid symbol 10, offset 348`. tauri-action used to scrub these silently.

## [0.1.6] - 2026-05-02

### Fixed
- `tauri.conf.json` now sets `bundle.createUpdaterArtifacts: true`. Without this, native `tauri build` does not produce `.sig` files or `.nsis.zip` / `.app.tar.gz` updater bundles, even when `TAURI_SIGNING_PRIVATE_KEY` is set. The aggregator job had nothing to stitch into `latest.json` in 0.1.5. (`tauri-apps/tauri-action` was passing the equivalent flag implicitly via `includeUpdaterJson`, which is why earlier attempts saw the sigs in the action's own log.)

## [0.1.5] - 2026-05-02

### Fixed
- Release workflow: replaced `tauri-apps/tauri-action@v0` with native `bun run tauri build` invocations. tauri-action's bundle outputs were silently inaccessible to follow-up steps, breaking signature collection on every previous attempt. Now the workflow owns the file paths end-to-end: it creates the draft, runs `tauri build`, copies bundles + sigs from `src-tauri/target/.../release/bundle/**`, uploads bundles to the release, and ships sigs to the aggregator job as artifacts.

## [0.1.4] - 2026-05-02

### Fixed
- Release workflow: re-add `includeUpdaterJson: true` so tauri-action actually generates the per-bundle `.sig` files. Without it, the aggregator job had nothing to stitch into `latest.json`.
- Manifest job: replace YAML-indented heredoc (which never matched its own `EOF` terminator) with a `jq -n` invocation. Failed silently in 0.1.3 even when sigs existed.
- Manifest job hard-fails with a clear error if any required signature is missing, instead of writing a half-empty `latest.json`.

## [0.1.3] - 2026-05-02

### Added
- Portable Windows distribution: `*_x64_portable.zip` containing the standalone `.exe` for users who don't want an installer.
- `latest.json` updater manifest is now uploaded to the release. The in-app auto-updater can finally see new versions.
- `.nsis.zip` / `.msi.zip` updater bundles uploaded as release assets so the updater can download them on Windows.

### Changed
- `release.yml` workflow split into a `build` matrix and a `manifest` aggregator job. The aggregator collects per-platform signature files as workflow artifacts, then constructs `latest.json` with `windows-x86_64`, `darwin-x86_64`, and `darwin-aarch64` entries.

## [0.1.2] - 2026-05-02

### Added
- macOS support via `--target universal-apple-darwin`. One `.dmg` works on both Apple Silicon and Intel.
- Site landing page under `site/` with prerendered EN / JA / KO pages auto-deployed to GitHub Pages on `site/**` push.
- macOS download card and FAQ updates on the landing site.

### Fixed
- Switched mac build off the `macos-13` Intel runner (queues 20+ minutes) onto the universal binary built on `macos-latest`.

## [0.1.0] - 2026-05-02

### Added
- Real-time channel-point redemption pickup via Twitch EventSub WebSocket.
- Playing / Waiting / Trash zones with drag-and-drop reordering.
- First-time-today badges and configurable first-timer prioritisation.
- Built-in HTTP server for OBS Browser Source overlay.
- Native overlay window that auto-fits its content height to remove the empty bottom strip.
- Seven themes (Twitch default, Midnight, Daylight, Sakura, Forest, High Contrast, Custom). Themed modal scrolls inside its own region so it fits inside the 450px minHeight window.
- Three languages: 日本語 / English / 한국어.
- Auto-updater wiring (manifest publishing fixed in 0.1.3).
- Local mock-server testing mode using the Twitch CLI WebSocket.

### Fixed
- Auth: refresh-token rotation race that logged users out across dev restarts. All refresh paths now serialise through a single mutex and reload tokens after acquiring the lock.
- Pipeline: `subscribe()` was using a stale access token after refresh; now uses the freshest value.
- Overlay window initial height was a fixed 460px regardless of content; switched to `ResizeObserver` + `setSize` driven by `scrollHeight`.

[Unreleased]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.2.5...HEAD
[0.2.5]: https://github.com/acerola/twitch-sankagata-manager/releases/tag/v0.2.5
[0.2.4]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.2.3...v0.2.4
[0.1.10]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/acerola/twitch-sankagata-manager/compare/v0.1.0...v0.1.2
[0.1.0]: https://github.com/acerola/twitch-sankagata-manager/releases/tag/v0.1.0
