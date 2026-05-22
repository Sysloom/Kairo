use std::sync::Mutex;

use crate::{
    domain::{
        models::{TimeInterval, TimerStatus},
        timer_engine::{ActiveTimerState, TimerEngine},
        timer_snapshot::TimerSnapshot,
    },
    infrastructure::{
        clock::SystemClock,
        repositories::{SqliteRepository, StepPersistenceStatus, StopReason},
    },
};

pub struct AppState {
    engine: Mutex<TimerEngine<SystemClock>>,
    repository: SqliteRepository,
}

impl AppState {
    pub fn new(repository: SqliteRepository) -> Self {
        Self {
            engine: Mutex::new(TimerEngine::new(SystemClock)),
            repository,
        }
    }

    pub fn snapshot(&self) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        Ok(engine.snapshot())
    }

    pub fn today_focus_ms(&self) -> Result<i64, String> {
        let now_ms = self.snapshot()?.now_ms;
        self.repository.today_focus_ms(now_ms)
    }

    pub fn start_free_session(&self, duration_ms: i64) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous = engine.active_state();
        let snapshot = engine.start_free_session(duration_ms);

        if let Some(previous) = previous {
            self.persist_cancelled_state(previous, snapshot.now_ms, StopReason::Reset)?;
        }
        self.persist_active_state(&engine, StepPersistenceStatus::Running, snapshot.now_ms)?;

        Ok(snapshot)
    }

    #[allow(dead_code)]
    pub fn start_focus_break_cycle(
        &self,
        focus_ms: i64,
        break_ms: i64,
        total_focus_target_ms: i64,
        total_cycle_target_ms: i64,
    ) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous = engine.active_state();
        let snapshot = engine.start_focus_break_cycle(
            focus_ms,
            break_ms,
            total_focus_target_ms,
            total_cycle_target_ms,
        );

        if let Some(previous) = previous {
            self.persist_cancelled_state(previous, snapshot.now_ms, StopReason::Reset)?;
        }
        self.persist_active_state(&engine, StepPersistenceStatus::Running, snapshot.now_ms)?;

        Ok(snapshot)
    }

    #[allow(dead_code)]
    pub fn continue_cycle(&self) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let snapshot = engine.continue_cycle();
        self.persist_active_state(&engine, StepPersistenceStatus::Running, snapshot.now_ms)?;

        Ok(snapshot)
    }

    pub fn pause(&self) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous_interval_id = engine
            .active_state()
            .and_then(|active| active.current_interval.map(|interval| interval.id));
        let snapshot = engine.pause();

        if let Some(interval_id) = previous_interval_id {
            if let Some(interval) = engine.interval(&interval_id) {
                self.repository
                    .close_interval(&interval, StopReason::Pause)?;
            }
        }
        self.persist_active_state(&engine, StepPersistenceStatus::Paused, snapshot.now_ms)?;

        Ok(snapshot)
    }

    pub fn resume(&self) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let snapshot = engine.resume();
        self.persist_active_state(&engine, StepPersistenceStatus::Running, snapshot.now_ms)?;

        Ok(snapshot)
    }

    pub fn reset(&self) -> Result<TimerSnapshot, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous = engine.active_state();
        let snapshot = engine.reset();

        if let Some(previous) = previous {
            self.persist_cancelled_state(previous, snapshot.now_ms, StopReason::Reset)?;
        }

        Ok(snapshot)
    }

    pub fn complete_if_due(&self) -> Result<Option<TimerSnapshot>, String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous_interval_id = engine
            .active_state()
            .and_then(|active| active.current_interval.map(|interval| interval.id));

        if engine.complete_if_due() {
            let snapshot = engine.snapshot();
            let close_reason = if matches!(snapshot.status, TimerStatus::Completed) {
                StopReason::Complete
            } else {
                StopReason::StepSwitch
            };
            if let Some(interval_id) = previous_interval_id {
                if let Some(interval) = engine.interval(&interval_id) {
                    self.repository.close_interval(&interval, close_reason)?;
                }
            }
            self.persist_active_state(&engine, StepPersistenceStatus::Completed, snapshot.now_ms)?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    pub fn quit(&self) -> Result<(), String> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| "timer engine lock was poisoned".to_string())?;

        let previous = engine.active_state();
        let snapshot = engine.reset();
        if let Some(previous) = previous {
            self.persist_cancelled_state(previous, snapshot.now_ms, StopReason::Quit)?;
        }
        self.repository
            .close_all_open_intervals(snapshot.now_ms, StopReason::Quit)?;

        Ok(())
    }

    pub fn close_for_app_shutdown(&self) -> Result<(), String> {
        let now_ms = self.snapshot()?.now_ms;
        self.repository
            .close_all_open_intervals(now_ms, StopReason::AppShutdown)
    }

    fn persist_active_state(
        &self,
        engine: &TimerEngine<SystemClock>,
        step_status: StepPersistenceStatus,
        updated_at_ms: i64,
    ) -> Result<(), String> {
        let Some(session) = engine.current_session() else {
            return Ok(());
        };
        let steps = engine.current_steps();
        if steps.is_empty() {
            return Ok(());
        }

        self.repository.upsert_session(&session)?;
        for step in steps {
            let persisted_status = match step.status {
                crate::domain::models::StepStatus::Pending => StepPersistenceStatus::Pending,
                crate::domain::models::StepStatus::Running => StepPersistenceStatus::Running,
                crate::domain::models::StepStatus::Paused => StepPersistenceStatus::Paused,
                crate::domain::models::StepStatus::Completed => StepPersistenceStatus::Completed,
                crate::domain::models::StepStatus::Cancelled => StepPersistenceStatus::Cancelled,
            };
            let status = if step.id
                == engine
                    .current_step()
                    .map(|current| current.id)
                    .unwrap_or_default()
            {
                step_status
            } else {
                persisted_status
            };
            let started_at_ms = if matches!(status, StepPersistenceStatus::Pending) {
                None
            } else {
                Some(session.started_at_ms)
            };
            let ended_at_ms = if matches!(
                status,
                StepPersistenceStatus::Completed | StepPersistenceStatus::Cancelled
            ) {
                Some(updated_at_ms)
            } else {
                session.ended_at_ms
            };
            self.repository.upsert_step(
                &step,
                status,
                started_at_ms,
                ended_at_ms,
                updated_at_ms,
            )?;
        }
        if let Some(interval) = engine
            .active_state()
            .and_then(|active| active.current_interval)
        {
            self.repository.open_interval(&interval)?;
        }

        Ok(())
    }

    fn persist_cancelled_state(
        &self,
        previous: ActiveTimerState,
        ended_at_ms: i64,
        stop_reason: StopReason,
    ) -> Result<(), String> {
        let ActiveTimerState {
            mut session,
            mut steps,
            current_interval,
        } = previous;

        if let Some(interval) = closed_interval(current_interval, ended_at_ms) {
            if let Some(step) = steps.iter_mut().find(|step| step.id == interval.step_id) {
                step.actual_ms = (step.actual_ms + interval.elapsed_ms).min(step.planned_ms);
            }
            self.repository.close_interval(&interval, stop_reason)?;
        }

        if !matches!(session.status, TimerStatus::Completed) {
            session.ended_at_ms = Some(ended_at_ms);
            self.repository.upsert_session(&session)?;
            for step in steps {
                let status = if matches!(step.status, crate::domain::models::StepStatus::Completed)
                {
                    StepPersistenceStatus::Completed
                } else {
                    StepPersistenceStatus::Cancelled
                };
                self.repository.upsert_step(
                    &step,
                    status,
                    Some(session.started_at_ms),
                    Some(ended_at_ms),
                    ended_at_ms,
                )?;
                if !matches!(status, StepPersistenceStatus::Completed) {
                    self.repository.mark_step_cancelled(&step.id, ended_at_ms)?;
                }
            }
            self.repository
                .mark_session_cancelled(&session.id, ended_at_ms)?;
        }

        Ok(())
    }
}

fn closed_interval(interval: Option<TimeInterval>, ended_at_ms: i64) -> Option<TimeInterval> {
    interval.map(|mut interval| {
        interval.ended_at_ms = Some(ended_at_ms);
        interval.elapsed_ms = (ended_at_ms - interval.started_at_ms).max(0);
        interval
    })
}
