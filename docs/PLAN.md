# Momentum — Implementation Plan

> Status: **approved (2026-11-26)**. Name: *Momentum*.
> Companion docs: [DESIGN.md](DESIGN.md) (visual system & screens), [DECISIONS.md](DECISIONS.md) (ADRs), [PROGRESS.md](PROGRESS.md).

Momentum is a cross-platform (Windows / macOS / Linux, x64 + native ARM64) focus timer built around
*intention → dial → focus → reflection → break*. Everything is local, free, and open source (MIT).

---

## 1. Goals and non-goals

**Goals**
- First session started in **< 10 s** after first launch, without reading anything.
- A precise, dark, "instrument" look: the dial is the signature component.
- Timer engine in Rust as the single source of truth; resilient to sleep, clock changes, restarts.
- Native builds for 6 targets, auto-update, reproducible README visuals.

**Non-goals (v1)**
- Accounts, servers, cloud sync (schema is *sync-ready* only).
- Mobile apps.
- Re-using any Session asset (logo, icon, illustrations, copy). We reproduce interaction ideas only.

---

## 2. Architecture

```
┌──────────────────────────────── Momentum desktop process (Tauri 2) ───────────────────────────────┐
│                                                                                                    │
│  WebView (React 18 + TS + Vite)            Rust core                                               │
│  ┌──────────────────────────┐   typed     ┌──────────────────────────────────────────────────────┐ │
│  │ main window              │  commands   │ momentum-core   (pure, no Tauri: state machine,      │ │
│  │  ├ Timer panel  (dial)   │◀──────────▶ │                  scheduler math, analytics, import)  │ │
│  │  └ Day calendar panel    │  + events   │ momentum-store  (SQLite, migrations, repositories)   │ │
│  │ analytics window/view    │ (specta)    │ app crate       (Tauri glue: commands, event bus,    │ │
│  │ settings window          │             │                  tray, windows, shortcuts, deep links│ │
│  │ mini player window       │             │                  notifications, audio, services)     │ │
│  └──────────────────────────┘             └──────────────┬───────────────────────────────────────┘ │
│                                                          │ internal event bus (tokio broadcast)    │
│        ┌──────────┬──────────┬───────────┬───────────┬───┴───────┬───────────┬──────────┐          │
│        ▼          ▼          ▼           ▼           ▼           ▼           ▼          ▼          │
│      Tray     Notifier    Audio       Site-block   App-block  Automations  Slack    Calendar       │
│   (dynamic   (native +   (rodio,     (WS server   (per-OS    (scripts,    (OAuth)  (ICS/Google/   │
│    icon)      actions)   generated)   127.0.0.1)   watcher)   webhooks)            Outlook)       │
└────────────────────────────────────────────────────┬───────────────────────────────────────────────┘
                 ▲ local IPC (named pipe / UDS)       │ WebSocket + pairing token
                 │                                    ▼
          momentum CLI (sidecar binary)        Browser extension (MV3, Chromium + Firefox)
          URL scheme momentum:// → single-instance forwarding
```

### 2.1 Repository layout (monorepo: Cargo workspace + pnpm workspace)

```
momentum/
├─ Cargo.toml                 # workspace
├─ package.json, pnpm-workspace.yaml
├─ apps/desktop/
│  ├─ src/                    # React app (features/, components/, dial/, stores/, lib/ipc.ts)
│  ├─ src-tauri/              # Tauri app crate (glue only), tauri.conf.json, capabilities/
│  └─ index.html, vite.config.ts, tailwind.config.ts
├─ crates/
│  ├─ momentum-core/          # domain types, timer state machine, scheduler, analytics, CSV/JSON import-export
│  ├─ momentum-store/         # rusqlite + migrations + repositories + backups
│  ├─ momentum-ipc/           # shared protocol: CLI ↔ app, extension ↔ app (serde types, versioned)
│  └─ momentum-cli/           # `momentum` CLI, bundled as a Tauri sidecar (externalBin)
├─ extension/                 # MV3 extension, one codebase, per-browser build (chromium, firefox)
├─ locales/{en,fr}/*.json     # single source of UI strings, also consumed by Rust (tray, notifications)
├─ assets/                    # icon sources, CC0 sounds, fonts (OFL), emoji (MIT)
├─ scripts/                   # demo-data, screenshots (Playwright), gif pipeline, release helpers
├─ docs/                      # PLAN, DESIGN, DECISIONS, PROGRESS, assets/ (README visuals)
└─ .github/workflows/         # ci.yml (lint/test), build.yml (6-target matrix), release.yml, visuals.yml
```

Rationale: `momentum-core` has **no Tauri dependency**, so the state machine and analytics are fast to unit-test
and reused by the CLI and by the demo-data generator.

### 2.2 Timer engine (Rust, source of truth)

- **Explicit state machine** (`enum TimerState`): `Idle`, `Breathing`, `Focus`, `Paused`, `FocusOvertime`,
  `Reflection`, `Break`, `LongBreak`, `BreakOvertime`, terminal `Abandoned` (undoable for 5 s).
  Transitions are a pure function `fn apply(state, command, now) -> Result<(state, Vec<Effect>)>`; all
  transitions covered by unit tests + a property test (random command sequences never produce invalid states).
- **Absolute timestamps only**: `started_at`, `planned_sec`, `pauses: Vec<Interval>`, `extensions`. Elapsed time is
  always derived: `elapsed = now - started_at - Σ pauses`. No incrementing counters.
- **Persistence**: the live timer is written to an `active_timer` singleton row on every transition (WAL mode, fsync
  on commit). On launch, the engine rehydrates and resumes — including the case where the deadline passed while the
  app was closed (→ lands directly in overtime with the correct `+mm:ss`).
- **Scheduler**: instead of ticking, the engine computes the *next deadline* (presence chime, pre-end warning, end,
  "went too long" repeats, reflection auto-validate) and sleeps until `min(deadlines)`. It re-evaluates on OS
  resume / clock-change notifications (and a 30 s safety re-check), so sleep and DST never drift.
- **Event bus**: transitions emit typed `TimerEvent`s (`session_start`, `session_pause`, `session_unpause`,
  `session_went_too_long`, `session_end`, `break_start`, `break_went_too_long`, `break_end`, `stop_working`, plus
  `snapshot_changed`). Consumers subscribe: UI (via `tauri::Emitter`), tray, notifier, audio, blockers,
  automations, Slack, calendar writer.
- **UI animation**: the frontend never receives per-second ticks; it receives a `TimerSnapshot` (timestamps) and
  animates with `requestAnimationFrame` from absolute time → smooth 60 fps, zero IPC chatter, and rAF stops when
  the window is hidden (CPU ≈ 0 %). The tray updates at most once per 10 s (ring) / 1 s (macOS text, optional).

### 2.3 Data layer

- SQLite via **`rusqlite` (bundled)** + **`rusqlite_migration`** (versioned, embedded SQL files), in the OS
  app-data dir (`dirs`/Tauri path API). WAL mode. All IDs **UUID v7**; every syncable table has
  `created_at`, `updated_at`, `deleted_at` (soft delete).
- Schema = prompt §7 plus: `active_timer` (singleton), `extension_pairing`, `automation_run` (logs),
  `task_cache` (imported to-dos), `window_tracker_rule`. Indexes on `session(started_at)`,
  `session(category_id, started_at)`, `session(deleted_at)`.
- **Analytics aggregates computed in Rust** (`momentum-core::analytics`), exposed as typed commands; TS types
  generated by **`tauri-specta`** (commands + events) → no hand-written IPC types.
- Time zones: sessions stored in UTC; bucketing by day/week/hour uses the user's *current* IANA zone via
  `jiff` (handles DST correctly). Sessions crossing midnight are split proportionally across days for daily
  totals (and shown as one block in timelines). Tested explicitly (midnight, DST forward/back, zone change).
- Daily backup with 7-day rotation (`VACUUM INTO`), restore from Settings > Data.

### 2.4 Frontend

- React 18 + TS + Vite, Tailwind (tokens mapped to CSS variables), Radix primitives, Framer Motion, Lucide.
- Zustand for UI state only; timer state is a read-only mirror of Rust snapshots.
- Routing: tiny in-app router (views: `main`, `analytics`, `settings/<section>`, `onboarding`, `mini`), driven by
  deep links too.
- i18next with `locales/en` as reference, `locales/fr`; ICU plurals; dates/durations via `Intl`.
- Charts: **visx** (low-level, fully token-driven) — see DECISIONS ADR-009.
- Fonts bundled locally (`@fontsource`): Inter, JetBrains Mono (OFL) — no network at runtime.

### 2.5 System integration summary (details in DECISIONS)

| Concern | Approach |
|---|---|
| Custom title bar | macOS: `titleBarStyle: Overlay` (native traffic lights on the left). Win/Linux: `decorations:false` + own controls on the right. |
| Vibrancy | macOS `NSVisualEffectView` (sidebar material), Windows 11 Mica, Linux solid fill. |
| Tray | Tauri tray; icon rendered at runtime with `tiny-skia` (progress ring in category color); macOS menu-bar title text. |
| Hide from Alt-Tab | Windows: tool-window style + skip taskbar; macOS: `ActivationPolicy::Accessory`; Linux: skip-taskbar hint (X11). |
| Notifications | `tauri-plugin-notification` for basic; actionable toasts via native backends (WinRT toasts, `UNUserNotificationCenter`, D-Bus `notify-rust`). |
| Audio | `rodio` in Rust (works with window hidden); noise generators procedural; CC0 samples embedded. |
| Single instance / deep links | `tauri-plugin-single-instance` + `tauri-plugin-deep-link` (`momentum://`). |
| CLI | `momentum` sidecar talks to the running app over a local socket (`interprocess`: named pipe / UDS). |
| Global shortcuts / autostart / updater | `tauri-plugin-global-shortcut`, `-autostart`, `-updater` (signed updates). |
| Secrets | OS keychain via `keyring`. |

---

## 3. Phases

Each phase ends with: compiles + launches on the 6 targets (CI), tests green, one clean commit (or a small
series), `docs/PROGRESS.md` updated. "Launches" in CI = a `--smoke-test` flag that boots the app, opens the DB,
creates the window, and exits 0 (Linux under `xvfb`); real-hardware checks are listed in PROGRESS.

### Phase 1 — Foundation
- Monorepo scaffold, Tauri 2 + React/TS/Vite, Tailwind wired to design tokens, fonts.
- Frameless window with custom title bar per OS, vibrancy where available, dark/light/system theme.
- i18n EN/FR (Rust + TS share `locales/`), language switch.
- SQLite + migration v1 (full §7 schema), app-data path, `tauri-specta` bindings generation.
- Tooling: rustfmt, clippy `-D warnings`, ESLint, Prettier, Vitest, `cargo test`.
- CI: lint/test on PR; build matrix → **5 jobs / 6 targets** (macOS universal covers 2), artifacts uploaded;
  smoke test per job; release workflow skeleton with signing slots (Apple, Authenticode, updater key) that are
  skipped when secrets are absent.
- ✅ Empty app launches on all targets; one artifact per target in CI.

### Phase 2 — Core timer
- `momentum-core` state machine + scheduler + persistence/rehydration; event bus.
- Dial (drag CW/CCW with unwrap, 1-min/5-min snapping, wheel, arrows ±1/±5, typed "45" / "1h30", optional
  tick sound), live drain at 60 fps, scrubbing during a session, overtime tint.
- Intention field + suggestions (recent/frequent, per category), categories CRUD (reorder/archive/delete,
  Ctrl/⌘+K picker), breathing phase, reflection (moods 1/2/3, notes, auto-validate countdown), breaks + long
  break every N, daily goal dots, basic sounds, basic notifications, abandon-with-undo, start previous.
- ✅ focus → reflection → break → focus fully by keyboard; session survives sleep and app restart.

### Phase 3 — Dual panel & day calendar
- Week strip, day header stats, category split bar, vertical timeline, live hatched block, now-line,
  drag/resize edit, detail panel, "+" manual add, collapsible panel, window size/position memory.
- ✅ Matches reference layouts 1 and 2.

### Phase 4 — Journal & analytics
- Demo-data generator (3 months, realistic), Day/Week/Month views, period list with sparklines, 24 h barcode,
  KPI cards with deltas, daily stacked histogram, category donut, weekday×hour heatmap, streaks, filters,
  full-text search (SQLite FTS5 over intent + notes), CSV/JSON export/import (incl. Session format), backups.
- ✅ With demo data, screens match references 4 and 5.

### Phase 5 — System presence
- Dynamic tray / menu bar, mini player window, global shortcuts, autostart, single instance, URL scheme,
  CLI sidecar, Do-Not-Disturb templates, hide from Dock/taskbar/Alt-Tab options.
- ✅ Everything controllable without opening the main window.

### Phase 6 — Blockers
- Extension (Chromium + Firefox builds), WS server + pairing, blocked page (intention + animated dial + time
  left), tab restore, allow-list mode, focus/break lists, emergency unblock (marks session distracted).
- App blocker per OS (Windows foreground hook, macOS NSWorkspace, X11 `_NET_ACTIVE_WINDOW`, Wayland fallback).
- ✅ YouTube blocked during focus with intention shown, tab restored at the end.

### Phase 7 — Integrations
- Calendars (ICS, Google, Outlook — read + optional write, ICS export), Slack (status + snooze, multi-workspace),
  to-dos (Todoist, Linear, GitHub, Markdown file, Things 3), automations (scripts, webhooks, templates, logs,
  "Test"), window tracker, full ambient sound mixer with crossfades and user imports.
- ✅ Each integration has a "Test" button and clear errors.

### Phase 8 — Polish & release
- Onboarding (≤ 4 screens), final motion pass, a11y audit (keyboard, ARIA, AA contrast, dial as slider),
  performance pass, auto-update end-to-end, final icon/branding, screenshot/GIF pipeline, showcase README,
  optional static download page.
- ✅ v1.0.0 release with 6-target binaries and showcase README.

Rough size (single developer + agent, for ordering only): P1 S · P2 L · P3 M · P4 L · P5 M · P6 L · P7 XL · P8 M.

---

## 4. Risks by area

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | **Wayland**: no portable way to read the focused window or hide other apps' windows. | App blocker & window tracker degraded on GNOME/KDE Wayland. | Detect session type; X11/XWayland path where possible; KWin script backend (KDE) as stretch; fallback = process polling with "quit" mode only. Clearly labeled in Settings. |
| R2 | **Code signing absent** → macOS Gatekeeper ("damaged"/unidentified developer) on Apple Silicon, Windows SmartScreen warnings. | Install friction. | Ad-hoc sign macOS builds (`signingIdentity: "-"`) so ARM launches; document `xattr`/right-click Open; slots for Developer ID + notarization and Authenticode; README install notes. |
| R3 | **Updater** requires a signing key pair; losing the private key breaks updates forever. | High, irreversible. | Generate once, store as GitHub secret + offline backup (your call, see Q3). |
| R4 | **Windows ARM64 MSI**: WiX-based MSI is not supported for ARM64 by the Tauri bundler. | Only NSIS `.exe` for ARM64. | Ship NSIS for ARM64, MSI + NSIS for x64. Documented. |
| R5 | **OAuth** Google Calendar scope is "sensitive": unverified app shows a warning and is capped at 100 users until Google verification. Slack requires an HTTPS redirect and a client secret for token exchange. | Integrations friction. | PKCE + loopback for Google/Microsoft; Microsoft public client (no secret). Slack: tiny static HTTPS redirect page (GitHub Pages) bouncing to `momentum://`, or "bring your own Slack app token" mode. "Bring your own client ID" setting everywhere. |
| R6 | **App blocker** semantics differ per OS (minimizing other apps' windows, overlays above fullscreen apps). macOS may require Accessibility permission for some actions. | Inconsistent UX. | Per-OS capability matrix in Settings; never terminate system/protected processes (deny-list + ownership check); "hide" is default, "quit" opt-in. |
| R7 | **Do Not Disturb / Focus Assist** has no public API on Windows; macOS Focus has no public toggle. | Feature partially "template-based". | Windows: open Focus settings / documented best-effort; macOS: Shortcuts-based template (`shortcuts run`); Linux: GNOME `gsettings` / KDE D-Bus. Delivered as automation templates, not core promises. |
| R8 | **Actionable notifications** need an installed app identity (AUMID on Windows, bundle on macOS). | Dev builds can't show actions. | Feature-detect; fall back to plain notifications + in-app/tray actions. |
| R9 | **MV3 service worker lifetime** may drop the WebSocket. | Blocker stops working mid-session. | Active WS keeps the worker alive on modern Chromium; heartbeat + reconnect; extension caches the last state (with expiry) so blocking survives brief disconnects. |
| R10 | **RAM < 120 MB**: WebView2/WebKit multi-process footprint is often above that in total. | Target may be unachievable when measured as total. | Measure both "app process" and "total incl. webview"; lazy-create analytics/settings/mini windows; destroy hidden secondary windows. Report honestly in PROGRESS. |
| R11 | **ARM runners** (`windows-11-arm`, `ubuntu-24.04-arm`) are free for public repos only. | CI cost for a private repo. | See Q2. |
| R12 | **Session import compatibility** without a real sample export. | Import may miss fields. | Implement from the documented field list; validate against a real export if you can provide one (Q5). |
| R13 | **Name "Momentum"** is already used by a well-known productivity browser extension (Momentum Dash). | Store rejection / confusion; bundle id and URL scheme are hard to change later. | Decide final name before phase 1 identifiers are set (Q1). |

---

## 5. Testing strategy

- **Rust**: unit tests for every transition; property tests (`proptest`) on command sequences; scheduler tests with
  an injected clock (sleep, DST, clock jumps, restart mid-pause); analytics tests (midnight split, DST days, zone
  change, empty periods, deltas); store tests against in-memory SQLite + migration up-tests.
- **Front**: Vitest + Testing Library for the dial math (angle ↔ duration, CW/CCW, unwrap across 12 o'clock,
  snapping, typed parsing "45", "1h30", "90m"), suggestions, reflection shortcuts.
- **E2E**: WebdriverIO + `tauri-driver` (Windows + Linux in CI; macOS has no WebDriver for WKWebView → covered by
  Playwright on the front with a mocked IPC layer): first full session, reflection, export.
- **Visual pipeline** (phase 8): Playwright against the front with a simulated Tauri layer + demo data.

---

## 6. Resolved questions (2026-11-26)

1. **Name**: keep *Momentum* (bundle id `app.momentum.desktop`, scheme `momentum://`, CLI `momentum`).
2. **Repository**: public, `alexroussel/momentum` → free native ARM runners.
3. **Updater key**: generated locally; private key stored as the `TAURI_SIGNING_PRIVATE_KEY` GitHub secret, with an
   offline backup kept by the owner.
4. **OAuth apps**: "bring your own client ID" until the owner registers Google / Microsoft / Slack apps (phase 7).
5. **Session export sample**: none available; import is implemented from the documented field list and tolerant
   to unknown/missing columns (R12 stays open).
