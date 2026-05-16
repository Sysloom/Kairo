use tauri::{AppHandle, State};

use crate::{app_state::AppState, domain::timer_snapshot::TimerSnapshot, infrastructure::windows};

#[tauri::command]
pub fn show_main_window(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    windows::show_main_window(&app)?;
    let snapshot = state.snapshot()?;
    windows::sync_mini_timer_window(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn hide_main_window(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    windows::hide_main_window(&app)?;
    let snapshot = state.snapshot()?;
    windows::sync_mini_timer_window(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn show_timer_window(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    windows::show_timer_window(&app)?;
    let snapshot = state.snapshot()?;
    windows::sync_mini_timer_window(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn hide_timer_window(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    windows::hide_timer_window(&app)?;
    let snapshot = state.snapshot()?;
    windows::sync_mini_timer_window(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.quit()?;
    app.exit(0);
    Ok(())
}
