mod app_state;
mod commands;
mod domain;
mod infrastructure;

use app_state::AppState;
use commands::{
    continue_timer_cycle, get_timer_snapshot, get_today_focus_ms, hide_main_window,
    hide_timer_window, pause_timer, quit_app, reset_timer, resume_timer, show_main_window,
    show_timer_window, start_focus_break_cycle, start_free_session,
};
use domain::models::TimerStatus;
use infrastructure::{db, kde_integration, notifications, tray, windows};
use std::time::Duration;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let repository = db::initialize(app.handle())?;
            app.manage(AppState::new(repository));
            tray::setup(app)?;
            kde_integration::setup(app.handle());

            if let Some(timer_window) = app.get_webview_window(windows::TIMER_WINDOW_LABEL) {
                windows::apply_timer_window_defaults(&timer_window);
            }

            if let Some(mini_timer_window) =
                app.get_webview_window(windows::MINI_TIMER_WINDOW_LABEL)
            {
                windows::apply_mini_timer_window_defaults(&mini_timer_window);
            }

            let app_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(500));
                let state = app_handle.state::<AppState>();
                match state.complete_if_due() {
                    Ok(Some(snapshot)) => {
                        if let Err(error) =
                            notifications::notify_step_completed(&app_handle, &snapshot)
                        {
                            eprintln!("failed to show step completion notification: {error}");
                        }
                        if let Err(error) = commands::emit_step_completed(&app_handle, &snapshot) {
                            eprintln!("failed to emit step completion: {error}");
                        }
                        match snapshot.status {
                            TimerStatus::AwaitingContinue => {
                                if let Err(error) =
                                    commands::emit_awaiting_continue(&app_handle, &snapshot)
                                {
                                    eprintln!("failed to emit awaiting continue: {error}");
                                }
                            }
                            TimerStatus::Completed => {
                                if let Err(error) =
                                    commands::emit_cycle_completed(&app_handle, &snapshot)
                                {
                                    eprintln!("failed to emit cycle completion: {error}");
                                }
                                if let Err(error) = commands::emit_completed(&app_handle, &snapshot)
                                {
                                    eprintln!("failed to emit legacy timer completion: {error}");
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {}
                    Err(error) => eprintln!("failed to check timer completion: {error}"),
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            windows::hide_window_on_close(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            get_timer_snapshot,
            get_today_focus_ms,
            start_free_session,
            start_focus_break_cycle,
            continue_timer_cycle,
            pause_timer,
            resume_timer,
            reset_timer,
            show_main_window,
            hide_main_window,
            show_timer_window,
            hide_timer_window,
            quit_app
        ])
        .build(tauri::generate_context!())
        .expect("error while building Kairo");

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let state = app_handle.state::<AppState>();
            if let Err(error) = state.close_for_app_shutdown() {
                eprintln!("failed to close open intervals during shutdown: {error}");
            }
        }
    });
}
