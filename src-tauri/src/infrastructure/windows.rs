use std::time::Duration;

use crate::{
    app_state::AppState,
    domain::{
        models::{StepKind, TimerStatus},
        timer_snapshot::TimerSnapshot,
    },
};

use tauri::{AppHandle, Manager, PhysicalSize, WebviewWindow, Window, WindowEvent};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const TIMER_WINDOW_LABEL: &str = "timer";
pub const MINI_TIMER_WINDOW_LABEL: &str = "mini-timer";
const MINI_TIMER_WIDTH: u32 = 80;
const MINI_TIMER_HEIGHT: u32 = 32;
const MINI_TIMER_FINAL_WARNING_MS: i64 = 5_000;

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    hide_mini_timer_window(app)?;
    show_window(app, MAIN_WINDOW_LABEL)
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    hide_window(app, MAIN_WINDOW_LABEL)?;
    sync_mini_timer_window_from_app_delayed(app.clone());
    Ok(())
}

pub fn show_timer_window(app: &AppHandle) -> Result<(), String> {
    hide_mini_timer_window(app)?;
    let window = window_by_label(app, TIMER_WINDOW_LABEL)?;
    apply_timer_window_defaults(&window);
    window.show().map_err(to_window_error)?;
    apply_timer_window_defaults(&window);
    window.set_focus().map_err(to_window_error)?;
    Ok(())
}

pub fn hide_timer_window(app: &AppHandle) -> Result<(), String> {
    hide_window(app, TIMER_WINDOW_LABEL)?;
    sync_mini_timer_window_from_app_delayed(app.clone());
    Ok(())
}

pub fn sync_mini_timer_window_from_app(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let snapshot = state.snapshot()?;

    sync_mini_timer_window(app, &snapshot)
}

pub fn sync_mini_timer_window(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    let primary_timer_windows_hidden =
        !is_window_visible(app, MAIN_WINDOW_LABEL)? && !is_window_visible(app, TIMER_WINDOW_LABEL)?;

    match mini_timer_action_for_snapshot(snapshot, primary_timer_windows_hidden) {
        MiniTimerAction::ShowMini => show_mini_timer_window(app),
        MiniTimerAction::ShowFloatingTimer => {
            hide_mini_timer_window(app)?;
            show_timer_window(app)
        }
        MiniTimerAction::HideMini => hide_mini_timer_window(app),
    }
}

pub fn hide_window_on_close(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let label = window.label().to_string();

        if let Err(error) = window.hide() {
            eprintln!("failed to hide window on close: {error}");
        }

        if let Err(error) = sync_mini_timer_window_from_app(window.app_handle()) {
            eprintln!("failed to sync mini timer after window close: {error}");
        }

        if label == MAIN_WINDOW_LABEL || label == TIMER_WINDOW_LABEL {
            sync_mini_timer_window_from_app_delayed(window.app_handle().clone());
        }
    }
}

pub fn apply_timer_window_defaults(window: &WebviewWindow) {
    apply_overlay_window_defaults(window, "timer");
}

pub fn apply_mini_timer_window_defaults(window: &WebviewWindow) {
    apply_overlay_window_defaults(window, "mini timer");
}

fn show_mini_timer_window(app: &AppHandle) -> Result<(), String> {
    let window = window_by_label(app, MINI_TIMER_WINDOW_LABEL)?;
    apply_mini_timer_window_defaults(&window);
    window
        .set_size(PhysicalSize::new(MINI_TIMER_WIDTH, MINI_TIMER_HEIGHT))
        .map_err(to_window_error)?;
    window.show().map_err(to_window_error)?;
    apply_mini_timer_window_defaults(&window);
    window.set_focus().map_err(to_window_error)
}

fn hide_mini_timer_window(app: &AppHandle) -> Result<(), String> {
    hide_window(app, MINI_TIMER_WINDOW_LABEL)
}

fn apply_overlay_window_defaults(window: &WebviewWindow, label: &str) {
    if let Err(error) = window.set_always_on_top(true) {
        eprintln!("failed to set {label} always-on-top: {error}");
    }

    if let Err(error) = window.set_skip_taskbar(true) {
        eprintln!("failed to skip {label} taskbar entry: {error}");
    }
}

fn is_active_timer_status(status: &TimerStatus) -> bool {
    matches!(
        status,
        TimerStatus::Running | TimerStatus::Paused | TimerStatus::AwaitingContinue
    )
}

#[derive(Debug, PartialEq, Eq)]
enum MiniTimerAction {
    ShowMini,
    ShowFloatingTimer,
    HideMini,
}

fn mini_timer_action_for_snapshot(
    snapshot: &TimerSnapshot,
    primary_timer_windows_hidden: bool,
) -> MiniTimerAction {
    if !primary_timer_windows_hidden
        || !is_active_timer_status(&snapshot.status)
        || !is_focus_step(snapshot)
    {
        return MiniTimerAction::HideMini;
    }

    if snapshot.status == TimerStatus::Running
        && snapshot.remaining_ms <= MINI_TIMER_FINAL_WARNING_MS
    {
        return MiniTimerAction::ShowFloatingTimer;
    }

    if snapshot.remaining_ms > MINI_TIMER_FINAL_WARNING_MS {
        MiniTimerAction::ShowMini
    } else {
        MiniTimerAction::HideMini
    }
}

fn is_focus_step(snapshot: &TimerSnapshot) -> bool {
    snapshot
        .current_step
        .as_ref()
        .is_some_and(|step| matches!(&step.kind, StepKind::Focus))
}

fn is_window_visible(app: &AppHandle, label: &str) -> Result<bool, String> {
    let window = window_by_label(app, label)?;
    window.is_visible().map_err(to_window_error)
}

fn show_window(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = window_by_label(app, label)?;
    window.show().map_err(to_window_error)?;
    window.set_focus().map_err(to_window_error)?;
    Ok(())
}

fn hide_window(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = window_by_label(app, label)?;
    window.hide().map_err(to_window_error)
}

fn sync_mini_timer_window_from_app_delayed(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));

        if let Err(error) = sync_mini_timer_window_from_app(&app) {
            eprintln!("failed to sync mini timer after delayed hide: {error}");
        }
    });
}

fn window_by_label(app: &AppHandle, label: &str) -> Result<WebviewWindow, String> {
    app.get_webview_window(label)
        .ok_or_else(|| format!("window `{label}` was not found"))
}

fn to_window_error(error: tauri::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::timer_snapshot::{TimerSnapshot, TimerSnapshotStep};

    #[test]
    fn mini_timer_shows_only_for_hidden_focus_with_more_than_five_seconds() {
        let snapshot = running_snapshot(StepKind::Focus, 6_000);

        assert_eq!(
            mini_timer_action_for_snapshot(&snapshot, true),
            MiniTimerAction::ShowMini
        );
    }

    #[test]
    fn mini_timer_does_not_show_for_break_steps() {
        let snapshot = running_snapshot(StepKind::ShortBreak, 60_000);

        assert_eq!(
            mini_timer_action_for_snapshot(&snapshot, true),
            MiniTimerAction::HideMini
        );
    }

    #[test]
    fn final_focus_seconds_restore_floating_timer() {
        let snapshot = running_snapshot(StepKind::Focus, 5_000);

        assert_eq!(
            mini_timer_action_for_snapshot(&snapshot, true),
            MiniTimerAction::ShowFloatingTimer
        );
    }

    #[test]
    fn mini_timer_hides_when_primary_windows_are_visible() {
        let snapshot = running_snapshot(StepKind::Focus, 60_000);

        assert_eq!(
            mini_timer_action_for_snapshot(&snapshot, false),
            MiniTimerAction::HideMini
        );
    }

    fn running_snapshot(kind: StepKind, remaining_ms: i64) -> TimerSnapshot {
        TimerSnapshot {
            status: TimerStatus::Running,
            current_step: Some(TimerSnapshotStep {
                id: "step-1".into(),
                kind,
                index: 0,
                planned_ms: 60_000,
                actual_ms: 60_000 - remaining_ms,
            }),
            remaining_ms,
            ..TimerSnapshot::idle(1_000)
        }
    }
}
