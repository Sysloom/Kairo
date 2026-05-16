# AGENTS.md — Kairo

## Product Snapshot

**Kairo** is a local-first desktop focus timer for **Fedora KDE / Plasma / Wayland** built with **Tauri v2 + React + TypeScript + Rust + SQLite**.

The app currently works as a KDE-first desktop product, not just a Phase 1 spike. It already includes:

- system tray control surface;
- main window;
- floating timer window;
- mini timer window for hidden-window countdowns;
- local SQLite persistence for sessions, steps, and intervals;
- light/dark theme support;
- alarm selection from packaged MP3 assets.

Visible app name: **Kairo**.

## Current Goal

Maintain and polish the current vertical slice without regressing the core architecture.

Priority order:

1. Preserve timer correctness.
2. Preserve safe tray/window lifecycle behavior.
3. Preserve KDE-first usability.
4. Keep docs, validation, and packaging aligned with the real app state.

## Non-Negotiable Architecture Rules

### Rust is the source of truth

Timer state and persisted focus truth live in Rust, not React.

React may:

- render snapshots from Rust;
- send user intent via Tauri commands;
- subscribe to Tauri events;
- do cosmetic UI refreshes based on the latest snapshot.

React must not:

- compute authoritative elapsed focus time;
- own the active session lifecycle;
- persist timer truth directly;
- use `setInterval` as the timer source of truth.

### Honest focus tracking only

Focus time is derived from real running intervals:

```text
interval.started_at_ms
interval.ended_at_ms
interval.elapsed_ms = ended_at_ms - started_at_ms
```

Paused time never counts.
Planned duration never counts unless it actually ran.

### Closing windows is not quitting

- Closing `main` hides it.
- Closing `timer` hides it.
- The app quits only through tray Quit or explicit quit command.
- Quitting must close and persist any open interval safely.

## Current Windows

### Main window (`main`)

- default size `456x676`
- min size `420x636`
- shows timer state, controls, settings, and simple daily stats

### Floating timer (`timer`)

- frameless
- transparent
- always-on-top
- skip taskbar
- supports compact and analog variants

### Mini timer (`mini-timer`)

- size `80x32`
- frameless
- transparent
- always-on-top
- skip taskbar
- only used for focus countdown when both other windows are hidden

## Mini Timer Rules

Mini timer is shown only when ALL are true:

- session is active;
- current step is `focus`;
- `main` is hidden;
- `timer` is hidden;
- remaining focus time is greater than 5 seconds.

Mini timer is NOT used for breaks.

During the final `<= 5s` of a running focus step:

- mini timer must hide;
- floating timer must open in the last selected variant.

## Theme Rules

- Supported themes: `light` and `dark`.
- Dark theme must stay warm, never pure black.
- Theme sync between windows must use Tauri IPC event `kairo://theme-changed`.
- Do not rely only on DOM-local events like `window.dispatchEvent` for cross-window sync.

## Branding Rules

- Root `logo/` directory is the source of truth for brand assets.
- Do not remove, rename, or relocate the source SVG assets.
- Tauri tray and bundle icons must use PNG derivatives under `src-tauri/icons/`.
- Keep the visible name as **Kairo** in user-facing surfaces.

## Tray Rules

The tray is a control surface and state indicator.

It must keep a contextual menu with at least:

- show/hide focus panel;
- show/hide floating timer;
- settings entry;
- start/pause/resume/reset controls;
- quit.

Do not use a cramped rectangular countdown rendered inside the KDE tray if it degrades into an unreadable square/compressed badge.

## KDE / Wayland Notes

- Always-on-top should be treated as best-effort under Wayland.
- KDE may need a manual KWin rule in some environments; this should not block the product.
- Keep the main window initial size larger than its min size. Starting exactly at min size caused a first-launch native close-button issue on KDE/Wayland.

## Current Stack

- Desktop runtime: Tauri v2
- Frontend: React + TypeScript + Vite
- Backend/core: Rust
- Database: SQLite via `rusqlite`
- State sync: Tauri commands + Tauri events
- Package manager: pnpm
- Target platform: Fedora KDE / Plasma / Wayland first

## Key Paths

```text
src/
  app/
  components/
  features/focus/
  services/
  styles/
  windows/

src-tauri/
  capabilities/default.json
  migrations/001_init.sql
  icons/
  src/
    commands/
    domain/
    infrastructure/
```

Important current files:

- `src/windows/MainWindow/MainWindow.tsx`
- `src/windows/TimerWindow/TimerWindow.tsx`
- `src/windows/MiniTimerWindow/MiniTimerWindow.tsx`
- `src/features/focus/themePreference.ts`
- `src/features/focus/floatingTimerPreference.ts`
- `src/services/audioService.ts`
- `src-tauri/src/domain/timer_engine.rs`
- `src-tauri/src/infrastructure/windows.rs`
- `src-tauri/src/infrastructure/tray.rs`
- `src-tauri/src/infrastructure/repositories.rs`

## Development Commands

```bash
npx pnpm@10.11.0 install
npx pnpm@10.11.0 dev
npx pnpm@10.11.0 tauri dev --no-watch
npx pnpm@10.11.0 lint
npx pnpm@10.11.0 typecheck
npx pnpm@10.11.0 build
```

Rust validation:

```bash
cargo fmt
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Packaging example:

```bash
npx pnpm@10.11.0 tauri build --bundles rpm
```

## Validation Discipline

Run checks sequentially.

Do **not** run `cargo test` in parallel with `pnpm build` or another process that mutates `dist/assets`, because Tauri/Rust tests can fail on stale or regenerated frontend bundle paths.

Recommended sequence:

1. `cargo fmt`
2. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
3. `cargo check --manifest-path src-tauri/Cargo.toml`
4. `cargo test --manifest-path src-tauri/Cargo.toml`
5. `npx pnpm@10.11.0 lint`
6. `npx pnpm@10.11.0 typecheck`
7. `npx pnpm@10.11.0 build`
8. `npx pnpm@10.11.0 tauri build --bundles rpm`

## Manual QA Checklist

- App launches on Fedora KDE.
- Tray icon appears.
- Tray menu can show/hide main window.
- Tray menu can show/hide floating timer.
- Starting focus opens a session and updates snapshots.
- Pause closes the active interval and does not count paused time.
- Resume opens a new interval.
- Reset cancels safely.
- Closing windows hides them instead of quitting.
- Mini timer appears only for hidden-window active focus with more than 5 seconds remaining.
- Final 5 seconds hand off from mini timer to floating timer.
- Dark/light theme syncs across all windows.
- Completion alarm plays and can be dismissed.
- Quit closes any open interval and persists state.

## Out of Scope Unless Explicitly Requested

Do not add by default:

- cloud sync;
- authentication;
- GNOME-specific work;
- global shortcuts;
- autostart;
- advanced analytics;
- export flows;
- packaging beyond what is needed for requested release artifacts.
