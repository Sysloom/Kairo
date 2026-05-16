use tauri::{AppHandle, Emitter, State};

use crate::{app_state::AppState, domain::timer_snapshot::TimerSnapshot, infrastructure::windows};

pub const TIMER_SNAPSHOT_EVENT: &str = "timer://snapshot";
pub const TIMER_COMPLETED_EVENT: &str = "timer://completed";
pub const TIMER_STEP_COMPLETED_EVENT: &str = "timer://step-completed";
pub const TIMER_AWAITING_CONTINUE_EVENT: &str = "timer://awaiting-continue";
pub const TIMER_CYCLE_COMPLETED_EVENT: &str = "timer://cycle-completed";
pub const TIMER_RESET_EVENT: &str = "timer://reset";

#[tauri::command]
pub fn get_timer_snapshot(state: State<'_, AppState>) -> Result<TimerSnapshot, String> {
    state.snapshot()
}

#[tauri::command]
pub fn get_today_focus_ms(state: State<'_, AppState>) -> Result<i64, String> {
    state.today_focus_ms()
}

#[tauri::command]
pub fn start_free_session(
    app: AppHandle,
    state: State<'_, AppState>,
    duration_ms: i64,
) -> Result<TimerSnapshot, String> {
    let snapshot = state.start_free_session(duration_ms)?;
    crate::infrastructure::windows::show_timer_window(&app)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn start_focus_break_cycle(
    app: AppHandle,
    state: State<'_, AppState>,
    focus_ms: i64,
    break_ms: i64,
    total_focus_target_ms: i64,
) -> Result<TimerSnapshot, String> {
    let snapshot = state.start_focus_break_cycle(focus_ms, break_ms, total_focus_target_ms)?;
    crate::infrastructure::windows::show_timer_window(&app)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn continue_timer_cycle(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    let snapshot = state.continue_cycle()?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn pause_timer(app: AppHandle, state: State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let snapshot = state.pause()?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn resume_timer(app: AppHandle, state: State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let snapshot = state.resume()?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn reset_timer(app: AppHandle, state: State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let snapshot = state.reset()?;
    app.emit(TIMER_RESET_EVENT, &snapshot)
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

pub fn emit_snapshot(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    windows::sync_mini_timer_window(app, snapshot)?;
    app.emit(TIMER_SNAPSHOT_EVENT, snapshot)
        .map_err(|error| error.to_string())
}

pub fn emit_completed(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    app.emit(TIMER_COMPLETED_EVENT, snapshot)
        .map_err(|error| error.to_string())?;
    emit_snapshot(app, snapshot)
}

pub fn emit_step_completed(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    app.emit(TIMER_STEP_COMPLETED_EVENT, snapshot)
        .map_err(|error| error.to_string())?;
    emit_snapshot(app, snapshot)
}

pub fn emit_awaiting_continue(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    app.emit(TIMER_AWAITING_CONTINUE_EVENT, snapshot)
        .map_err(|error| error.to_string())
}

pub fn emit_cycle_completed(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    app.emit(TIMER_CYCLE_COMPLETED_EVENT, snapshot)
        .map_err(|error| error.to_string())
}
