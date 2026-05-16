# Kairo

Kairo is a local-first focus timer desktop app for **Fedora KDE / Plasma / Wayland** built with **Tauri v2, React, TypeScript, Rust, and SQLite**.

It is designed around one non-negotiable rule: **Rust owns the real timer state and persisted focus truth**.

Spanish version: [README.es.md](./README.es.md)

## Download RPM

**Fedora KDE 44 build:**

- RPM: [Kairo-0.1.1-1.x86_64.rpm](https://github.com/AngelGAVargas/Kairo/releases/download/v0.1.1/Kairo-0.1.1-1.x86_64.rpm)

## Quick start

1. Install dependencies.
2. Run the desktop app in development mode.
3. Validate frontend and Rust checks before shipping.
4. Build an RPM package for Fedora KDE users.

## What the app includes today

- system tray integration;
- main control window;
- floating timer window;
- mini timer for hidden-window focus countdowns;
- local SQLite persistence;
- focus session pause/resume/reset flow;
- light and dark themes;
- packaged MP3 alarms with audible fallback.

## Recent fixes in this build

- packaged MP3 alarms now play correctly in the installed app;
- floating timer windows use stable KDE/KWin-matchable titles;
- KDE/Wayland installs now bundle a KWin script that Kairo auto-enables for the current user so floating timers stay above other windows;
- the RPM linked above was built for **Fedora KDE 44**.

## Architecture rules

| Area | Decision |
| --- | --- |
| Timer source of truth | Rust owns timer state, session lifecycle, and persisted intervals |
| Frontend responsibility | React renders snapshots and sends user intent only |
| Focus accounting | Count only real running intervals, never planned time |
| Window lifecycle | Closing windows hides them, quitting is explicit |
| Branding | `logo/` holds source SVG assets, `src-tauri/icons/` holds PNG bundle/tray assets |
| Theme sync | Cross-window sync uses `kairo://theme-changed` IPC |

## Project structure

```text
src/
  app/
  components/
  features/focus/
  services/
  styles/
  windows/

src-tauri/
  capabilities/
  icons/
  migrations/
  src/
    commands/
    domain/
    infrastructure/
```

## Development

### Requirements

- Node.js compatible with the installed pnpm toolchain
- `pnpm@10.11.0`
- Rust toolchain
- Tauri Linux system dependencies for WebKitGTK/AppIndicator on Fedora KDE

### Install

```bash
npx pnpm@10.11.0 install
```

### Run in browser-only frontend mode

```bash
npx pnpm@10.11.0 dev
```

### Run the desktop app

```bash
npx pnpm@10.11.0 tauri dev --no-watch
```

`--no-watch` is useful when you want tighter control over rebuild timing.

## Validation

Run these commands **sequentially**.

Do not run `cargo test` at the same time as `pnpm build`, because Tauri tests can fail if `dist/assets` is being regenerated.

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npx pnpm@10.11.0 lint
npx pnpm@10.11.0 typecheck
npx pnpm@10.11.0 build
```

## Packaging for Fedora KDE

### Build an RPM

```bash
npx pnpm@10.11.0 tauri build --bundles rpm
```

Expected output location:

```text
src-tauri/target/release/bundle/rpm/
```

### Install as a normal Fedora user

Recommended for **Fedora KDE 44**:

```bash
sudo dnf install -y "https://github.com/AngelGAVargas/Kairo/releases/download/v0.1.1/Kairo-0.1.1-1.x86_64.rpm"
```

If your Fedora desktop allows local package installation through Discover or another graphical installer, you can open the generated `.rpm` file directly.

CLI option:

```bash
sudo dnf install ./src-tauri/target/release/bundle/rpm/<package-name>.rpm
```

If the package is already installed and you rebuilt it:

```bash
sudo dnf reinstall ./src-tauri/target/release/bundle/rpm/<package-name>.rpm
```

If KDE still shows a generic icon right after installation, refresh the desktop launcher cache and reopen the launcher:

```bash
kbuildsycoca6 --noincremental
```

## KDE / Wayland notes

- `alwaysOnTop` is best-effort under Wayland.
- Kairo packages a KWin script and auto-installs it for the current user on KDE/Wayland so floating timer windows stay above other windows from the compositor side.
- The main window starts larger than its minimum size on purpose. Starting exactly at the minimum size caused a first-launch native close-button issue on KDE/Wayland.
- A tray-rendered rectangular countdown was intentionally rejected because KDE/Tauri compresses it into a poor unreadable shape.

### Force floating timers above other windows on KDE

On some Plasma Wayland sessions, Tauri's `alwaysOnTop` hint is ignored. The RPM includes the KWin script and Kairo installs/enables it for the current user at startup when it detects KDE/Wayland.

Manual fallback for development builds or troubleshooting:

```bash
npx pnpm@10.11.0 kde:install-kwin-script
```

Then restart Kairo and test the floating timer over another app.

The script matches only Kairo overlay timer windows:

- `Kairo Floating Timer`
- `Kairo Mini Timer`

It forces `keepAbove` and keeps those windows out of the taskbar, pager, and window switcher.

## Manual QA checklist

### Tray and windows

- [ ] Tray icon appears after launch.
- [ ] Tray menu shows/hides the main panel.
- [ ] Tray menu shows/hides the floating timer.
- [ ] Closing `main` hides instead of quitting.
- [ ] Closing `timer` hides instead of quitting.

### Timer behavior

- [ ] Start a free focus session.
- [ ] Pause closes the active interval.
- [ ] Resume opens a new interval.
- [ ] Reset cancels safely.
- [ ] Completion marks the session completed.
- [ ] Paused time is not counted as focus time.

### Mini timer

- [ ] Hide both `main` and `timer` during an active focus session.
- [ ] Confirm the mini timer appears only when more than 5 seconds remain.
- [ ] Confirm the mini timer never appears during breaks.
- [ ] Confirm the final 5 seconds switch from mini timer to floating timer.

### Theme and audio

- [ ] Switch between light and dark theme from settings.
- [ ] Confirm all windows receive the theme change.
- [ ] Confirm the selected alarm plays to completion.
- [ ] Confirm completion audio can be dismissed.

### Persistence and quit safety

- [ ] Start a session, then quit from tray.
- [ ] Confirm open intervals are closed safely.
- [ ] Reopen the app and verify persisted state/statistics remain valid.

## Important files

- `AGENTS.md` — project operating rules for agents/maintainers
- `src-tauri/tauri.conf.json` — windows, build hooks, bundling
- `src-tauri/src/domain/timer_engine.rs` — timer truth
- `src-tauri/src/infrastructure/windows.rs` — window lifecycle and mini timer rules
- `src-tauri/src/infrastructure/tray.rs` — tray behavior
- `src/features/focus/themePreference.ts` — theme persistence and IPC sync
- `src/services/audioService.ts` — alarm playback and dismissal
- `packaging/kde/kwin/kairo-keep-above/` — optional KWin script for reliable timer keep-above behavior on KDE/Wayland

## Current packaging note

The preferred distro-native artifact for Fedora KDE is **RPM**. If RPM bundling is unavailable on a given machine, use the next supported Tauri Linux artifact and document the limitation instead of hiding it.
