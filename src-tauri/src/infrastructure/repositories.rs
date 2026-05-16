use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::domain::models::{Session, SessionStep, StepKind, TimeInterval, TimerMode, TimerStatus};

#[derive(Clone)]
pub struct SqliteRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteRepository {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
        }
    }

    pub fn run_migrations(&self) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute_batch(include_str!("../../migrations/001_init.sql"))
            .map_err(|error| format!("failed to run SQLite migrations: {error}"))
    }

    pub fn upsert_session(&self, session: &Session) -> Result<(), String> {
        let status = session_status_value(&session.status);
        let mode = timer_mode_value(&session.mode);
        let updated_at_ms = session.ended_at_ms.unwrap_or(session.started_at_ms);
        let connection = self.connection()?;

        connection
            .execute(
                "INSERT INTO sessions (
                    id, mode, target_focus_ms, status, started_at_ms, ended_at_ms,
                    created_at_ms, updated_at_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?7)
                ON CONFLICT(id) DO UPDATE SET
                    target_focus_ms = excluded.target_focus_ms,
                    status = excluded.status,
                    ended_at_ms = excluded.ended_at_ms,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    session.id,
                    mode,
                    session.target_focus_ms,
                    status,
                    session.started_at_ms,
                    session.ended_at_ms,
                    updated_at_ms,
                ],
            )
            .map_err(|error| format!("failed to upsert session `{}`: {error}", session.id))?;

        Ok(())
    }

    pub fn mark_session_cancelled(&self, session_id: &str, ended_at_ms: i64) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE sessions
                 SET status = 'cancelled', ended_at_ms = COALESCE(ended_at_ms, ?2), updated_at_ms = ?2
                 WHERE id = ?1 AND status != 'completed'",
                params![session_id, ended_at_ms],
            )
            .map_err(|error| format!("failed to cancel session `{session_id}`: {error}"))?;
        Ok(())
    }

    pub fn upsert_step(
        &self,
        step: &SessionStep,
        status: StepPersistenceStatus,
        started_at_ms: Option<i64>,
        ended_at_ms: Option<i64>,
        updated_at_ms: i64,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO session_steps (
                    id, session_id, step_index, kind, planned_ms, actual_ms, status,
                    started_at_ms, ended_at_ms, created_at_ms, updated_at_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
                ON CONFLICT(id) DO UPDATE SET
                    actual_ms = excluded.actual_ms,
                    status = excluded.status,
                    ended_at_ms = excluded.ended_at_ms,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    step.id,
                    step.session_id,
                    step.index,
                    step_kind_value(&step.kind),
                    step.planned_ms,
                    step.actual_ms,
                    status.as_str(),
                    started_at_ms,
                    ended_at_ms,
                    updated_at_ms,
                ],
            )
            .map_err(|error| format!("failed to upsert step `{}`: {error}", step.id))?;
        Ok(())
    }

    pub fn mark_step_cancelled(&self, step_id: &str, ended_at_ms: i64) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE session_steps
                 SET status = 'cancelled', ended_at_ms = COALESCE(ended_at_ms, ?2), updated_at_ms = ?2
                 WHERE id = ?1 AND status != 'completed'",
                params![step_id, ended_at_ms],
            )
            .map_err(|error| format!("failed to cancel step `{step_id}`: {error}"))?;
        Ok(())
    }

    pub fn open_interval(&self, interval: &TimeInterval) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO time_intervals (
                    id, session_id, step_id, kind, status, started_at_ms, ended_at_ms,
                    elapsed_ms, stop_reason, created_at_ms, updated_at_ms
                ) VALUES (?1, ?2, ?3, ?4, 'open', ?5, NULL, 0, NULL, ?5, ?5)
                ON CONFLICT(id) DO NOTHING",
                params![
                    interval.id,
                    interval.session_id,
                    interval.step_id,
                    interval_kind_value(&interval.kind),
                    interval.started_at_ms,
                ],
            )
            .map_err(|error| format!("failed to open interval `{}`: {error}", interval.id))?;
        Ok(())
    }

    pub fn close_interval(
        &self,
        interval: &TimeInterval,
        stop_reason: StopReason,
    ) -> Result<(), String> {
        let Some(ended_at_ms) = interval.ended_at_ms else {
            return Ok(());
        };
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE time_intervals
                 SET status = 'closed', ended_at_ms = ?2, elapsed_ms = ?3,
                     stop_reason = ?4, updated_at_ms = ?2
                 WHERE id = ?1",
                params![
                    interval.id,
                    ended_at_ms,
                    interval.elapsed_ms,
                    stop_reason.as_str(),
                ],
            )
            .map_err(|error| format!("failed to close interval `{}`: {error}", interval.id))?;
        Ok(())
    }

    pub fn close_all_open_intervals(
        &self,
        ended_at_ms: i64,
        stop_reason: StopReason,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE time_intervals
                 SET status = 'closed', ended_at_ms = ?1,
                     elapsed_ms = MAX(0, ?1 - started_at_ms),
                     stop_reason = ?2, updated_at_ms = ?1
                 WHERE status = 'open'",
                params![ended_at_ms, stop_reason.as_str()],
            )
            .map_err(|error| format!("failed to close open intervals: {error}"))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn today_focus_ms(&self, now_ms: i64) -> Result<i64, String> {
        let day_start_ms = now_ms - now_ms.rem_euclid(86_400_000);
        let day_end_ms = day_start_ms + 86_400_000;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT COALESCE(SUM(elapsed_ms), 0)
                 FROM time_intervals
                 WHERE kind = 'focus'
                   AND status = 'closed'
                   AND started_at_ms >= ?1
                   AND started_at_ms < ?2",
                params![day_start_ms, day_end_ms],
                |row| row.get(0),
            )
            .map_err(|error| format!("failed to query today's focus total: {error}"))
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.connection
            .lock()
            .map_err(|_| "SQLite connection lock was poisoned".to_string())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepPersistenceStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Cancelled,
}

impl StepPersistenceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Copy)]
pub enum StopReason {
    Pause,
    Complete,
    Reset,
    Quit,
    AppShutdown,
    StepSwitch,
}

impl StopReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Complete => "complete",
            Self::Reset => "reset",
            Self::Quit => "quit",
            Self::AppShutdown => "app_shutdown",
            Self::StepSwitch => "step_switch",
        }
    }
}

fn timer_mode_value(mode: &TimerMode) -> &'static str {
    match mode {
        TimerMode::Free => "free",
    }
}

fn session_status_value(status: &TimerStatus) -> &'static str {
    match status {
        TimerStatus::Running => "running",
        TimerStatus::Paused => "paused",
        TimerStatus::AwaitingContinue => "paused",
        TimerStatus::Completed => "completed",
        TimerStatus::Idle => "cancelled",
    }
}

fn step_kind_value(kind: &StepKind) -> &'static str {
    match kind {
        StepKind::Focus => "focus",
        StepKind::ShortBreak => "short_break",
    }
}

fn interval_kind_value(kind: &StepKind) -> &'static str {
    match kind {
        StepKind::Focus => "focus",
        StepKind::ShortBreak => "break",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> SqliteRepository {
        let repository = SqliteRepository::new(Connection::open_in_memory().unwrap());
        repository.run_migrations().unwrap();
        repository
    }

    #[test]
    fn migration_supports_interval_lifecycle_and_today_total() {
        let repository = repository();
        let session = Session {
            id: "session-1".into(),
            mode: TimerMode::Free,
            target_focus_ms: 1_000,
            status: TimerStatus::Running,
            started_at_ms: 86_400_000,
            ended_at_ms: None,
        };
        let step = SessionStep {
            id: "step-1".into(),
            session_id: session.id.clone(),
            index: 0,
            kind: StepKind::Focus,
            status: crate::domain::models::StepStatus::Running,
            planned_ms: 1_000,
            actual_ms: 1_000,
        };
        let mut interval = TimeInterval {
            id: "interval-1".into(),
            session_id: session.id.clone(),
            step_id: step.id.clone(),
            kind: StepKind::Focus,
            started_at_ms: 86_400_100,
            ended_at_ms: None,
            elapsed_ms: 0,
        };

        repository.upsert_session(&session).unwrap();
        repository
            .upsert_step(
                &step,
                StepPersistenceStatus::Running,
                Some(session.started_at_ms),
                None,
                session.started_at_ms,
            )
            .unwrap();
        repository.open_interval(&interval).unwrap();
        interval.ended_at_ms = Some(86_401_100);
        interval.elapsed_ms = 1_000;
        repository
            .close_interval(&interval, StopReason::Complete)
            .unwrap();

        assert_eq!(repository.today_focus_ms(86_402_000).unwrap(), 1_000);
    }

    #[test]
    fn maps_short_break_steps_break_intervals_and_step_switch_reason() {
        let repository = repository();
        let session = Session {
            id: "session-2".into(),
            mode: TimerMode::Free,
            target_focus_ms: 2_000,
            status: TimerStatus::AwaitingContinue,
            started_at_ms: 100,
            ended_at_ms: None,
        };
        let step = SessionStep {
            id: "step-break".into(),
            session_id: session.id.clone(),
            index: 1,
            kind: StepKind::ShortBreak,
            status: crate::domain::models::StepStatus::Completed,
            planned_ms: 500,
            actual_ms: 500,
        };
        let mut interval = TimeInterval {
            id: "interval-break".into(),
            session_id: session.id.clone(),
            step_id: step.id.clone(),
            kind: StepKind::ShortBreak,
            started_at_ms: 200,
            ended_at_ms: None,
            elapsed_ms: 0,
        };

        repository.upsert_session(&session).unwrap();
        repository
            .upsert_step(
                &step,
                StepPersistenceStatus::Completed,
                Some(200),
                Some(700),
                700,
            )
            .unwrap();
        repository.open_interval(&interval).unwrap();
        interval.ended_at_ms = Some(700);
        interval.elapsed_ms = 500;
        repository
            .close_interval(&interval, StopReason::StepSwitch)
            .unwrap();

        let connection = repository.connection().unwrap();
        let step_kind: String = connection
            .query_row(
                "SELECT kind FROM session_steps WHERE id = ?1",
                params![step.id],
                |row| row.get(0),
            )
            .unwrap();
        let (interval_kind, stop_reason): (String, String) = connection
            .query_row(
                "SELECT kind, stop_reason FROM time_intervals WHERE id = ?1",
                params![interval.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(step_kind, "short_break");
        assert_eq!(interval_kind, "break");
        assert_eq!(stop_reason, "step_switch");
    }
}
