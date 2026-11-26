# Architecture Decision Records

Format: context → options → decision → consequences. All ADRs below were accepted with the plan on 2026-11-26 unless marked otherwise.

---

## ADR-001 — Monorepo with a Tauri-free core crate
- **Context**: state machine, analytics and import/export must be heavily unit-tested and reused by the CLI and
  the demo-data generator.
- **Options**: (a) everything in `src-tauri`; (b) Cargo workspace with `momentum-core`, `momentum-store`,
  `momentum-ipc`, `momentum-cli` + thin Tauri app crate.
- **Decision**: (b). pnpm workspace for `apps/desktop` and `extension`.
- **Consequences**: fast `cargo test` without WebView deps; slightly more boilerplate.

## ADR-002 — Timer driven by absolute timestamps and deadline scheduling
- **Options**: (a) 1 Hz tick incrementing counters; (b) absolute timestamps + compute next deadline.
- **Decision**: (b). Persist the active timer on each transition; rehydrate on launch; re-evaluate on OS resume,
  clock change, and a 30 s safety check.
- **Consequences**: robust to sleep/DST/restart; UI animates locally with rAF from the snapshot.

## ADR-003 — SQLite with `rusqlite` (bundled) + `rusqlite_migration`
- **Options**: `sqlx` (async, compile-time checked queries needing a DB at build time), `rusqlite` (sync, simple,
  bundled SQLite incl. FTS5), `diesel`.
- **Decision**: `rusqlite` with `bundled` feature, one connection behind a dedicated worker thread, WAL mode.
- **Consequences**: identical SQLite version on all 6 targets (no system lib issues on ARM Linux); FTS5 available.

## ADR-004 — UUID v7 + soft delete for sync readiness
- **Decision**: all primary keys UUID v7 (`uuid` crate, `v7` feature) stored as TEXT; `created_at`, `updated_at`,
  `deleted_at` on syncable tables; timestamps stored as UTC RFC 3339 / unix ms.

## ADR-005 — Typed IPC with `tauri-specta`
- **Decision**: all commands and events declared in Rust and exported to `apps/desktop/src/bindings.ts` on debug
  builds; CI fails if bindings are stale.

## ADR-006 — Browser extension transport: local WebSocket + pairing token
- **Options**:
  - *Native Messaging*: no open port, but needs a host manifest installed per browser per OS (registry keys on
    Windows, different paths for each Chromium fork and Firefox), launches a separate host process, and breaks
    silently when the app moves/updates.
  - *Local WebSocket*: one server on `127.0.0.1:<fixed port>` (default 47821, configurable), works identically in
    every browser, survives app updates.
- **Decision**: WebSocket bound to loopback only, with a pairing token shown in the app (6-digit code exchanged
  for a long random token stored in extension storage), `Origin` check (`chrome-extension://` /
  `moz-extension://` IDs allowed after pairing), versioned protocol in `momentum-ipc`, heartbeat + reconnect.
- **Consequences**: port collision possible → configurable port + fallback range; no admin rights needed.

## ADR-007 — CLI as a sidecar binary over a local socket
- **Context**: Windows GUI-subsystem executables can't write to the console, so the main binary can't double as
  a CLI.
- **Decision**: separate `momentum` CLI (`clap`) bundled via `externalBin`, talking to the running app through
  `interprocess` local sockets (named pipe `\\.\pipe\momentum-<user>`, UDS in the runtime dir). If the app isn't
  running, the CLI launches it and retries. URL-scheme calls go through `tauri-plugin-single-instance`.

## ADR-008 — Audio in Rust (`rodio`)
- **Options**: Web Audio in the WebView vs native audio.
- **Decision**: `rodio` in Rust: plays while windows are hidden or destroyed, keeps the timer → sound path in one
  process, procedural white/pink/brown noise generators, embedded CC0 samples (`include_bytes!`), crossfades.
- **Consequences**: Linux build needs `libasound2-dev`; small binary size increase.

## ADR-009 — Charts with visx
- **Options**: Recharts (fast to start, opinionated look), visx (primitives, full control).
- **Decision**: visx + hand-made SVG for the 24 h barcode and sparklines; all colors from tokens.

## ADR-010 — Tray icon rendered at runtime
- **Decision**: draw the ring/dial icon with `tiny-skia` into RGBA at 1x/2x, in the category color; template
  (monochrome) variant for the macOS menu bar with the remaining time as tray title. Re-render ≤ every 10 s or on
  state change.

## ADR-011 — Window chrome per OS
- **Decision**: macOS `titleBarStyle: "Overlay"` + hidden title (native traffic lights, left); Windows/Linux
  `decorations: false` with custom controls on the right and `data-tauri-drag-region`; Windows 11 Mica and macOS
  vibrancy via Tauri `windowEffects`, solid fill elsewhere (and when "reduce transparency" is on).

## ADR-012 — Shared locale files for Rust and TS
- **Decision**: `locales/<lang>/*.json` (i18next format) are the single source; Rust embeds them and resolves keys
  for tray menus and notifications with a minimal interpolation helper. English is the reference locale; a CI
  check fails on missing keys in other locales.

## ADR-013 — App blocker strategy per OS
| OS | Detection | Action |
|---|---|---|
| Windows | `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` → PID → exe path | minimize (`ShowWindow SW_MINIMIZE`) or graceful close (`WM_CLOSE`), overlay window |
| macOS | `NSWorkspace.didActivateApplicationNotification` (`objc2`) | `hide()` or `terminate()` (never `forceTerminate`), overlay |
| Linux X11 | `_NET_ACTIVE_WINDOW` via `x11rb` | iconify (`_NET_WM_STATE_HIDDEN` / `XIconifyWindow`) or `_NET_CLOSE_WINDOW` |
| Linux Wayland | no portable API; process polling (`sysinfo`) | "quit" mode only (SIGTERM to user-owned processes); KWin script backend as stretch goal |
- Hard deny-list of system processes (explorer.exe, Finder, Dock, loginwindow, gnome-shell, plasmashell, kwin…),
  and only processes owned by the current user are ever touched.

## ADR-014 — Windows ARM64 installers: NSIS only
- **Context**: the Tauri bundler's MSI (WiX) target doesn't support ARM64.
- **Decision**: x64 → MSI + NSIS; ARM64 → NSIS. Updater uses NSIS on both.

## ADR-015 — Unsigned builds must still launch
- **Decision**: macOS ad-hoc signature (`signingIdentity: "-"`) so Apple Silicon launches unsigned builds after
  the Gatekeeper prompt; release workflow conditionally signs/notarizes when `APPLE_*` secrets exist and
  Authenticode-signs when `WINDOWS_CERTIFICATE*` exist. Updater artifacts always signed with the Tauri key.

## ADR-016 — OAuth for Google / Microsoft / Slack
- **Decision**: Authorization Code + PKCE with a loopback redirect (`http://127.0.0.1:<random>/callback`) for
  Google ("Desktop app" client) and Microsoft (public client, `common` tenant). Slack requires HTTPS redirects and
  a client secret for the token exchange, which a local-only app cannot keep secret. **v1 ships "bring your own
  Slack app"**: the user creates a Slack app from a manifest we provide and pastes the user token. A one-click
  flow would need a tiny hosted token-exchange function (out of scope unless you want it). Tokens stored with `keyring`. All providers support "bring your own client ID".

## ADR-017 — Do Not Disturb as automation templates
- **Decision**: no reliable public API on Windows (Focus Assist) or macOS (Focus). Ship templates: macOS
  `shortcuts run "Momentum Focus On/Off"` (user installs provided shortcuts), Linux GNOME `gsettings set
  org.gnome.desktop.notifications show-banners false`, KDE D-Bus `org.freedesktop.Notifications.Inhibit`,
  Windows: best effort (documented limitations), opens Focus settings otherwise.

## ADR-018 — Mood emoji assets
- **Decision**: bundle Fluent Emoji (MIT) SVGs for the 3 moods so they render identically on every OS.

## ADR-019 — CI smoke test
- **Decision**: the app accepts `--smoke-test`: boots, runs migrations against a temp DB, creates the main window,
  waits for the frontend "ready" event, exits 0. Run on every build job (Linux under `xvfb-run`).

## ADR-020 — Updater signing key custody
- **Decision**: the minisign key pair was generated on the owner's machine (2026-11-26). The **public** key is
  committed at `apps/desktop/src-tauri/updater.key.pub` and will be embedded in the updater plugin config
  (phase 8). The private key and its password never enter the repository; they are stored as the GitHub secrets
  `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` plus an offline backup.
- Release builds enable `createUpdaterArtifacts` through `src-tauri/tauri.release.conf.json` only, so PR builds
  don't need the secret.

## ADR-021 — Linux builds on Ubuntu 22.04 runners
- **Context**: the prompt suggested `ubuntu-24.04-arm`. Binaries linked on a newer glibc don't run on older
  distributions.
- **Decision**: build both Linux targets on `ubuntu-22.04` / `ubuntu-22.04-arm` (glibc 2.35) for wider
  compatibility; tests can still run on newer images.

## ADR-022 — Tailwind CSS v4 and TypeScript 6.0
- Tailwind v4 is configured CSS-first (`@theme inline` in `src/styles/index.css` mapping our CSS variables), so
  there is no `tailwind.config.ts`.
- TypeScript is pinned to 6.0 because `typescript-eslint` 8 does not support TypeScript 7 yet.

## ADR-023 — Windows test executables embed the app manifest
- `tauri-build` only embeds the Common Controls v6 manifest into binaries; test executables then crash with
  `STATUS_ENTRYPOINT_NOT_FOUND`. `build.rs` disables the default manifest and embeds
  `windows-app-manifest.xml` into every link target.

## ADR-024 — macOS runner image
- `macos-14` is no longer available on GitHub-hosted runners (job rejected: "configuration not supported").
  Builds use `macos-15` (Apple Silicon); the universal binary is still cross-built for x86_64.
