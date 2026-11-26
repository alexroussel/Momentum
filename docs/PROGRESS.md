# Progress

## Current phase: 1 — Foundation

### Done
- Plan approved (2026-11-26); open questions resolved (see PLAN.md §6).
- Monorepo: Cargo workspace (`momentum-core`, `momentum-store`, Tauri app crate) + pnpm workspace.
- `momentum-core`: stable domain enums (session kind/result, mood, theme, category status), UUID v7 ids,
  category palette.
- `momentum-store`: SQLite (bundled, WAL), migration v1 with the full schema (incl. FTS5 search, active timer
  singleton, sync-ready columns), settings and categories repositories, default "Focus" category.
- Tauri 2 app: frameless window with custom controls (Windows/Linux), overlay title bar (macOS), Mica (Win 11) /
  vibrancy (macOS) with translucent background tokens, typed commands via `tauri-specta`, `--smoke-test` mode.
- Frontend: React 18 + TS + Vite 8 + Tailwind v4 wired to the design tokens (dark/light/system), bundled Inter and
  JetBrains Mono, i18n EN/FR sharing `locales/` with Rust, locale key check script, placeholder home screen.
- Tooling: rustfmt, clippy `-D warnings`, ESLint, Prettier, Vitest.
- CI: `ci.yml` (lint/test/bindings check), `build.yml` (5 jobs → 6 targets, smoke test, artifacts),
  `release.yml` (draft release, updater signatures, Apple/Authenticode slots).
- Updater key pair generated (private key outside the repo).

### Verified locally (Windows 11 x64)
- `cargo test --workspace`, clippy, fmt: green. Vitest: green. ESLint/typecheck: green.
- Smoke test of a production-mode build: `smoke test: ok` in ~0.8 s.
- Dark/light themes and EN/FR switching checked visually in the browser preview.

### Verified in CI (run 45791361195)
- All 5 build jobs green with smoke test passing: macOS universal, Windows x64, Windows ARM64, Linux x64,
  Linux ARM64. Artifacts: .dmg, .msi + .exe (x64), .exe (ARM64), .AppImage/.deb/.rpm (x64 + ARM64).

### Not yet verified
- Real-hardware launch on macOS, Linux and ARM64 machines (CI smoke tests only).
- RAM/startup budgets (measured in phase 8).

### Notes
- `pnpm dev` uses port 1420 (required by Tauri). Another local project may already hold it.

### Next — Phase 2 (core timer)
- State machine + scheduler + persistence in `momentum-core`, the dial, intention + suggestions, categories,
  breathing, overtime, reflection, breaks, daily goal, basic sounds and notifications.
