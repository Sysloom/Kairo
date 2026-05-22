use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
    #[serde(rename = "awaiting_continue")]
    AwaitingContinue,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerMode {
    Free,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Focus,
    ShortBreak,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct CycleConfig {
    pub focus_ms: i64,
    pub break_ms: i64,
    pub total_focus_target_ms: i64,
    pub total_cycle_target_ms: i64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Session {
    pub id: String,
    pub mode: TimerMode,
    pub target_focus_ms: i64,
    pub status: TimerStatus,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SessionStep {
    pub id: String,
    pub session_id: String,
    pub index: u32,
    pub kind: StepKind,
    pub status: StepStatus,
    pub planned_ms: i64,
    pub actual_ms: i64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TimeInterval {
    pub id: String,
    pub session_id: String,
    pub step_id: String,
    pub kind: StepKind,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub elapsed_ms: i64,
}
