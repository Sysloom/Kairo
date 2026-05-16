use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::domain::{
    models::StepKind,
    timer_snapshot::{TimerSnapshot, TimerSnapshotCtaKind},
};

pub fn notify_step_completed(app: &AppHandle, snapshot: &TimerSnapshot) -> Result<(), String> {
    let completed_step = snapshot
        .current_step
        .as_ref()
        .map(|step| match step.kind {
            StepKind::Focus => "Enfoque completado",
            StepKind::ShortBreak => "Descanso completado",
        })
        .unwrap_or("Etapa completada");

    let body = match snapshot.cta.kind {
        TimerSnapshotCtaKind::StartBreak => "Cuando estés listo, iniciá tu descanso.",
        TimerSnapshotCtaKind::StartFocus => "Cuando quieras, iniciá el próximo enfoque.",
        TimerSnapshotCtaKind::Restart => "Ciclo de enfoque completado.",
        _ => "Etapa del temporizador completada.",
    };

    app.notification()
        .builder()
        .title(completed_step)
        .body(body)
        .show()
        .map_err(|error| format!("failed to show step completion notification: {error}"))
}
