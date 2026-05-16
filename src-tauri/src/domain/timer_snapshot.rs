use serde::Serialize;

use super::models::{StepKind, TimerMode, TimerStatus};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub session_id: Option<String>,
    pub mode: Option<TimerMode>,
    pub status: TimerStatus,
    pub current_step: Option<TimerSnapshotStep>,
    pub next_step: Option<TimerSnapshotStep>,
    pub cta: TimerSnapshotCta,
    pub cycle: Option<TimerSnapshotCycle>,
    pub started_at_ms: Option<i64>,
    pub current_interval_started_at_ms: Option<i64>,
    pub now_ms: i64,
    pub remaining_ms: i64,
    pub total_focus_elapsed_ms: i64,
    pub total_focus_target_ms: i64,
    pub active_kind_elapsed_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshotStep {
    pub id: String,
    pub kind: StepKind,
    pub index: u32,
    pub planned_ms: i64,
    pub actual_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshotCycle {
    pub focus_ms: i64,
    pub break_ms: i64,
    pub total_focus_target_ms: i64,
    pub completed_focus_ms: i64,
    pub completed_steps: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshotCta {
    pub kind: TimerSnapshotCtaKind,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimerSnapshotCtaKind {
    StartCycle,
    Pause,
    Resume,
    StartBreak,
    StartFocus,
    Restart,
}

impl TimerSnapshot {
    pub fn idle(now_ms: i64) -> Self {
        Self {
            session_id: None,
            mode: None,
            status: TimerStatus::Idle,
            current_step: None,
            next_step: None,
            cta: TimerSnapshotCta {
                kind: TimerSnapshotCtaKind::StartCycle,
                label: "Start focus".into(),
            },
            cycle: None,
            started_at_ms: None,
            current_interval_started_at_ms: None,
            now_ms,
            remaining_ms: 0,
            total_focus_elapsed_ms: 0,
            total_focus_target_ms: 0,
            active_kind_elapsed_ms: 0,
        }
    }
}
