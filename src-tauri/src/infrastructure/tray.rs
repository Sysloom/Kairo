use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    app_state::AppState,
    commands,
    domain::{
        models::{StepKind, TimerStatus},
        timer_snapshot::{TimerSnapshot, TimerSnapshotCtaKind},
    },
    infrastructure::windows,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

const TRAY_ID: &str = "focus-tray";
const STATUS_ID: &str = "status";
const TOGGLE_MAIN_ID: &str = "toggle-main";
const TOGGLE_TIMER_ID: &str = "toggle-timer";
const SETTINGS_ID: &str = "settings";
const PLAY_PAUSE_ID: &str = "play-pause";
const RESET_ID: &str = "reset";
const QUIT_ID: &str = "quit";
const SHOW_SETTINGS_EVENT: &str = "kairo://show-settings";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrayVisualState {
    Default,
    Active,
}

pub fn setup(app: &mut App) -> Result<(), String> {
    let status = MenuItem::with_id(app, STATUS_ID, "Kairo listo", false, None::<&str>)
        .map_err(to_menu_error)?;
    let toggle_main = MenuItem::with_id(app, TOGGLE_MAIN_ID, "Ocultar panel", true, None::<&str>)
        .map_err(to_menu_error)?;
    let toggle_timer = MenuItem::with_id(
        app,
        TOGGLE_TIMER_ID,
        "Mostrar timer flotante",
        true,
        None::<&str>,
    )
    .map_err(to_menu_error)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Configuración", true, None::<&str>)
        .map_err(to_menu_error)?;
    let play_pause = MenuItem::with_id(app, PLAY_PAUSE_ID, "▶ Iniciar 25/5", true, None::<&str>)
        .map_err(to_menu_error)?;
    let reset = MenuItem::with_id(app, RESET_ID, "Reiniciar sesión actual", true, None::<&str>)
        .map_err(to_menu_error)?;
    let quit =
        MenuItem::with_id(app, QUIT_ID, "Salir", true, None::<&str>).map_err(to_menu_error)?;

    let menu = Menu::with_items(
        app,
        &[
            &status,
            &toggle_main,
            &toggle_timer,
            &settings,
            &play_pause,
            &reset,
            &quit,
        ],
    )
    .map_err(to_menu_error)?;

    let tray_visual_state = Arc::new(Mutex::new(TrayVisualState::Default));

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Kairo")
        .icon(tray_icon_for_state(TrayVisualState::Default))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(error) = windows::show_main_window(tray.app_handle()) {
                    eprintln!("failed to show main window from tray click: {error}");
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            TOGGLE_MAIN_ID => {
                if let Err(error) = toggle_window_visibility(app, windows::MAIN_WINDOW_LABEL) {
                    eprintln!("failed to toggle main window from tray: {error}");
                }
            }
            TOGGLE_TIMER_ID => {
                if let Err(error) = toggle_window_visibility(app, windows::TIMER_WINDOW_LABEL) {
                    eprintln!("failed to toggle timer window from tray: {error}");
                }
            }
            SETTINGS_ID => {
                if let Err(error) = show_settings_from_tray(app) {
                    eprintln!("failed to show settings from tray: {error}");
                }
            }
            PLAY_PAUSE_ID => {
                if let Err(error) = run_primary_tray_action(app) {
                    eprintln!("failed to run tray primary action: {error}");
                }
            }
            RESET_ID => {
                let state = app.state::<AppState>();
                match state.reset() {
                    Ok(snapshot) => {
                        if let Err(error) = app.emit(commands::TIMER_RESET_EVENT, &snapshot) {
                            eprintln!("failed to emit reset from tray: {error}");
                        }
                        if let Err(error) = commands::emit_snapshot(app, &snapshot) {
                            eprintln!("failed to emit snapshot after tray reset: {error}");
                        }
                    }
                    Err(error) => eprintln!("failed to reset from tray: {error}"),
                }
            }
            QUIT_ID => {
                let state = app.state::<AppState>();
                if let Err(error) = state.quit() {
                    eprintln!("failed to persist timer state before tray quit: {error}");
                }
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(to_tray_error)?;

    spawn_tray_sync_loop(
        app.handle().clone(),
        status.clone(),
        toggle_main.clone(),
        toggle_timer.clone(),
        play_pause.clone(),
        reset.clone(),
        tray_visual_state.clone(),
    );

    let controls = TrayControls {
        status,
        toggle_main,
        toggle_timer,
        play_pause,
        reset,
        tray_visual_state,
    };

    if let Ok(snapshot) = app.state::<AppState>().snapshot() {
        update_tray(&app.handle().clone(), &controls, &snapshot);
    }

    Ok(())
}

fn spawn_tray_sync_loop(
    app: tauri::AppHandle,
    status: MenuItem<tauri::Wry>,
    toggle_main: MenuItem<tauri::Wry>,
    toggle_timer: MenuItem<tauri::Wry>,
    play_pause: MenuItem<tauri::Wry>,
    reset: MenuItem<tauri::Wry>,
    tray_visual_state: Arc<Mutex<TrayVisualState>>,
) {
    let controls = TrayControls {
        status,
        toggle_main,
        toggle_timer,
        play_pause,
        reset,
        tray_visual_state,
    };

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));

        let state = app.state::<AppState>();
        match state.snapshot() {
            Ok(snapshot) => {
                update_tray(&app, &controls, &snapshot);

                if let Err(error) = windows::sync_mini_timer_window(&app, &snapshot) {
                    eprintln!("failed to sync mini timer from tray loop: {error}");
                }
            }
            Err(error) => eprintln!("failed to sync tray timer state: {error}"),
        }
    });
}

struct TrayControls {
    status: MenuItem<tauri::Wry>,
    toggle_main: MenuItem<tauri::Wry>,
    toggle_timer: MenuItem<tauri::Wry>,
    play_pause: MenuItem<tauri::Wry>,
    reset: MenuItem<tauri::Wry>,
    tray_visual_state: Arc<Mutex<TrayVisualState>>,
}

fn update_tray(app: &tauri::AppHandle, controls: &TrayControls, snapshot: &TimerSnapshot) {
    let status_text = tray_status_text(snapshot);
    let tooltip_text = tray_tooltip_text(snapshot);

    if let Err(error) = controls.status.set_text(&status_text) {
        eprintln!("failed to update tray status text: {error}");
    }

    if let Err(error) =
        controls
            .toggle_main
            .set_text(toggle_window_text(app, windows::MAIN_WINDOW_LABEL, "panel"))
    {
        eprintln!("failed to update tray main window text: {error}");
    }

    if let Err(error) = controls.toggle_timer.set_text(toggle_window_text(
        app,
        windows::TIMER_WINDOW_LABEL,
        "timer flotante",
    )) {
        eprintln!("failed to update tray timer window text: {error}");
    }

    if let Err(error) = controls
        .play_pause
        .set_text(tray_primary_action_text(snapshot))
    {
        eprintln!("failed to update tray primary action text: {error}");
    }

    if let Err(error) = controls.reset.set_enabled(can_reset(snapshot)) {
        eprintln!("failed to update tray reset enabled state: {error}");
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Err(error) = tray.set_tooltip(Some(&tooltip_text)) {
            eprintln!("failed to update tray tooltip: {error}");
        }

        if should_update_tray_icon(&controls.tray_visual_state, snapshot) {
            if let Err(error) = tray.set_icon(Some(tray_icon_for_state(
                TrayVisualState::from_snapshot(snapshot),
            ))) {
                eprintln!("failed to update tray icon: {error}");
            }
        }
    }
}

impl TrayVisualState {
    fn from_snapshot(snapshot: &TimerSnapshot) -> Self {
        if matches!(
            snapshot.status,
            TimerStatus::Running | TimerStatus::Paused | TimerStatus::AwaitingContinue
        ) {
            Self::Active
        } else {
            Self::Default
        }
    }
}

fn should_update_tray_icon(
    tray_visual_state: &Arc<Mutex<TrayVisualState>>,
    snapshot: &TimerSnapshot,
) -> bool {
    let next_state = TrayVisualState::from_snapshot(snapshot);

    match tray_visual_state.lock() {
        Ok(mut current_state) => {
            if *current_state == next_state {
                false
            } else {
                *current_state = next_state;
                true
            }
        }
        Err(error) => {
            eprintln!("failed to lock tray visual state: {error}");
            true
        }
    }
}

fn tray_icon_for_state(state: TrayVisualState) -> Image<'static> {
    match state {
        TrayVisualState::Default => tauri::include_image!("icons/kairo-app-icon-ink.png"),
        TrayVisualState::Active => tauri::include_image!("icons/kairo-app-icon-accent.png"),
    }
}

fn toggle_window_visibility(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    let is_visible = is_window_visible(app, label)?;

    match (label, is_visible) {
        (windows::MAIN_WINDOW_LABEL, true) => windows::hide_main_window(app),
        (windows::MAIN_WINDOW_LABEL, false) => windows::show_main_window(app),
        (windows::TIMER_WINDOW_LABEL, true) => windows::hide_timer_window(app),
        (windows::TIMER_WINDOW_LABEL, false) => windows::show_timer_window(app),
        _ => Err(format!("window `{label}` cannot be toggled from tray")),
    }?;

    windows::sync_mini_timer_window_from_app(app)
}

fn show_settings_from_tray(app: &tauri::AppHandle) -> Result<(), String> {
    windows::show_main_window(app)?;
    windows::sync_mini_timer_window_from_app(app)?;
    app.emit(SHOW_SETTINGS_EVENT, ()).map_err(to_emit_error)
}

fn toggle_window_text(app: &tauri::AppHandle, label: &str, noun: &str) -> String {
    if is_window_visible(app, label).unwrap_or(false) {
        format!("Ocultar {noun}")
    } else {
        format!("Mostrar {noun}")
    }
}

fn is_window_visible(app: &tauri::AppHandle, label: &str) -> Result<bool, String> {
    app.get_webview_window(label)
        .ok_or_else(|| format!("window `{label}` was not found"))?
        .is_visible()
        .map_err(to_window_error)
}

fn run_primary_tray_action(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let snapshot = state.snapshot()?;
    let next_snapshot = match snapshot.status {
        TimerStatus::Idle | TimerStatus::Completed => {
            state.start_focus_break_cycle(25 * 60 * 1000, 5 * 60 * 1000, 50 * 60 * 1000)
        }
        TimerStatus::Running => state.pause(),
        TimerStatus::Paused => state.resume(),
        TimerStatus::AwaitingContinue => state.continue_cycle(),
    }?;

    commands::emit_snapshot(app, &next_snapshot)?;
    Ok(())
}

fn tray_status_text(snapshot: &TimerSnapshot) -> String {
    match snapshot.status {
        TimerStatus::Idle => "Kairo listo".into(),
        TimerStatus::Completed => "✅ Ciclo completo".into(),
        TimerStatus::Running => format!(
            "⏱ {} · {}",
            format_duration(snapshot.remaining_ms),
            step_label(snapshot)
        ),
        TimerStatus::Paused => format!("⏸ {} · pausado", format_duration(snapshot.remaining_ms)),
        TimerStatus::AwaitingContinue => next_step_label(snapshot),
    }
}

fn tray_tooltip_text(snapshot: &TimerSnapshot) -> String {
    match snapshot.status {
        TimerStatus::Idle => "Kairo".into(),
        TimerStatus::Completed => "Kairo · ciclo completo".into(),
        TimerStatus::Running | TimerStatus::Paused => {
            format!(
                "Kairo · {} restantes",
                format_duration(snapshot.remaining_ms)
            )
        }
        TimerStatus::AwaitingContinue => format!("Kairo · {}", next_step_label(snapshot)),
    }
}

fn tray_primary_action_text(snapshot: &TimerSnapshot) -> &'static str {
    match snapshot.status {
        TimerStatus::Idle => "▶ Iniciar 25/5",
        TimerStatus::Running => "⏸ Pausar",
        TimerStatus::Paused => "▶ Reanudar",
        TimerStatus::AwaitingContinue => match snapshot.cta.kind {
            TimerSnapshotCtaKind::StartBreak => "▶ Iniciar descanso",
            TimerSnapshotCtaKind::StartFocus => "▶ Iniciar foco",
            _ => "▶ Continuar",
        },
        TimerStatus::Completed => "▶ Reiniciar 25/5",
    }
}

fn can_reset(snapshot: &TimerSnapshot) -> bool {
    !matches!(snapshot.status, TimerStatus::Idle | TimerStatus::Completed)
}

fn next_step_label(snapshot: &TimerSnapshot) -> String {
    match snapshot.next_step.as_ref().map(|step| &step.kind) {
        Some(StepKind::ShortBreak) => "Sigue: descanso".into(),
        Some(StepKind::Focus) => "Sigue: enfoque".into(),
        None => "Listo para reiniciar".into(),
    }
}

fn step_label(snapshot: &TimerSnapshot) -> &'static str {
    match snapshot.current_step.as_ref().map(|step| &step.kind) {
        Some(StepKind::ShortBreak) => "descanso",
        _ => "enfoque",
    }
}

fn format_duration(duration_ms: i64) -> String {
    let total_seconds = (duration_ms / 1000).max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn to_menu_error(error: tauri::Error) -> String {
    error.to_string()
}

fn to_tray_error(error: tauri::Error) -> String {
    error.to_string()
}

fn to_window_error(error: tauri::Error) -> String {
    error.to_string()
}

fn to_emit_error(error: tauri::Error) -> String {
    error.to_string()
}
