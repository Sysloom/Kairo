use uuid::Uuid;

use super::{
    models::{
        CycleConfig, Session, SessionStep, StepKind, StepStatus, TimeInterval, TimerMode,
        TimerStatus,
    },
    timer_snapshot::{
        TimerSnapshot, TimerSnapshotCta, TimerSnapshotCtaKind, TimerSnapshotCycle,
        TimerSnapshotStep,
    },
};
use crate::infrastructure::clock::Clock;

pub struct TimerEngine<C: Clock> {
    clock: C,
    session: Option<Session>,
    steps: Vec<SessionStep>,
    active_step_index: Option<usize>,
    intervals: Vec<TimeInterval>,
    current_interval_id: Option<String>,
    cycle_config: Option<CycleConfig>,
}

#[derive(Clone, Debug)]
pub struct ActiveTimerState {
    pub session: Session,
    pub steps: Vec<SessionStep>,
    pub current_interval: Option<TimeInterval>,
}

impl<C: Clock> TimerEngine<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            session: None,
            steps: Vec::new(),
            active_step_index: None,
            intervals: Vec::new(),
            current_interval_id: None,
            cycle_config: None,
        }
    }

    pub fn snapshot(&mut self) -> TimerSnapshot {
        self.snapshot_without_transition()
    }

    pub fn start_free_session(&mut self, duration_ms: i64) -> TimerSnapshot {
        self.start_focus_break_cycle(duration_ms, 0, duration_ms)
    }

    pub fn start_focus_break_cycle(
        &mut self,
        focus_ms: i64,
        break_ms: i64,
        total_focus_target_ms: i64,
    ) -> TimerSnapshot {
        if matches!(
            self.status(),
            TimerStatus::Running | TimerStatus::Paused | TimerStatus::AwaitingContinue
        ) {
            self.reset_active();
        }

        self.steps.clear();
        self.intervals.clear();
        self.current_interval_id = None;

        let config = CycleConfig {
            focus_ms: focus_ms.max(1),
            break_ms: break_ms.max(1),
            total_focus_target_ms: total_focus_target_ms.max(1),
        };
        let now_ms = self.clock.now_ms();
        let session_id = Uuid::new_v4().to_string();
        let step_id = Uuid::new_v4().to_string();

        self.session = Some(Session {
            id: session_id.clone(),
            mode: TimerMode::Free,
            target_focus_ms: config.total_focus_target_ms,
            status: TimerStatus::Running,
            started_at_ms: now_ms,
            ended_at_ms: None,
        });
        self.steps.push(SessionStep {
            id: step_id.clone(),
            session_id: session_id.clone(),
            index: 0,
            kind: StepKind::Focus,
            status: StepStatus::Running,
            planned_ms: config.focus_ms,
            actual_ms: 0,
        });
        self.active_step_index = Some(0);
        self.cycle_config = Some(config);
        self.open_interval(now_ms, session_id, step_id, StepKind::Focus);

        self.snapshot_without_transition()
    }

    pub fn continue_cycle(&mut self) -> TimerSnapshot {
        if !matches!(self.status(), TimerStatus::AwaitingContinue) {
            return self.snapshot();
        }

        let Some(next_kind) = self.next_step_kind() else {
            return self.snapshot();
        };
        let Some(config) = self.cycle_config.clone() else {
            return self.snapshot();
        };
        let Some(session) = self.session.as_mut() else {
            return self.snapshot();
        };

        let now_ms = self.clock.now_ms();
        let step_id = Uuid::new_v4().to_string();
        let session_id = session.id.clone();
        let planned_ms = match next_kind {
            StepKind::Focus => config.focus_ms,
            StepKind::ShortBreak => config.break_ms,
        };

        session.status = TimerStatus::Running;
        self.steps.push(SessionStep {
            id: step_id.clone(),
            session_id: session_id.clone(),
            index: self.steps.len() as u32,
            kind: next_kind.clone(),
            status: StepStatus::Running,
            planned_ms,
            actual_ms: 0,
        });
        self.active_step_index = Some(self.steps.len() - 1);
        self.open_interval(now_ms, session_id, step_id, next_kind);

        self.snapshot_without_transition()
    }

    pub fn pause(&mut self) -> TimerSnapshot {
        if !matches!(self.status(), TimerStatus::Running) {
            return self.snapshot();
        }

        let now_ms = self.clock.now_ms();
        self.close_current_interval(now_ms);
        if let Some(session) = &mut self.session {
            session.status = TimerStatus::Paused;
        }
        if let Some(step) = self.current_step_mut() {
            step.status = StepStatus::Paused;
        }

        self.snapshot_without_transition()
    }

    pub fn resume(&mut self) -> TimerSnapshot {
        if !matches!(self.status(), TimerStatus::Paused) {
            return self.snapshot();
        }

        let now_ms = self.clock.now_ms();
        let Some(session_id) = self.session.as_ref().map(|session| session.id.clone()) else {
            return self.snapshot();
        };
        let Some((step_id, kind)) = self
            .current_step()
            .map(|step| (step.id.clone(), step.kind.clone()))
        else {
            return self.snapshot();
        };

        if let Some(session) = &mut self.session {
            session.status = TimerStatus::Running;
        }
        if let Some(step) = self.current_step_mut() {
            step.status = StepStatus::Running;
        }
        self.open_interval(now_ms, session_id, step_id, kind);

        self.snapshot_without_transition()
    }

    pub fn reset(&mut self) -> TimerSnapshot {
        self.reset_active();
        TimerSnapshot::idle(self.clock.now_ms())
    }

    pub fn active_state(&self) -> Option<ActiveTimerState> {
        Some(ActiveTimerState {
            session: self.session.clone()?,
            steps: self.steps.clone(),
            current_interval: self
                .current_interval_id
                .as_ref()
                .and_then(|id| self.intervals.iter().find(|interval| &interval.id == id))
                .cloned(),
        })
    }

    pub fn current_session(&self) -> Option<Session> {
        self.session.clone()
    }

    pub fn current_step(&self) -> Option<SessionStep> {
        self.current_step_ref().cloned()
    }

    pub fn current_steps(&self) -> Vec<SessionStep> {
        self.steps.clone()
    }

    pub fn interval(&self, interval_id: &str) -> Option<TimeInterval> {
        self.intervals
            .iter()
            .find(|interval| interval.id == interval_id)
            .cloned()
    }

    pub fn complete_if_due(&mut self) -> bool {
        if !matches!(self.status(), TimerStatus::Running) {
            return false;
        }

        let now_ms = self.clock.now_ms();
        let planned_ms = self
            .current_step_ref()
            .map(|step| step.planned_ms)
            .unwrap_or(0);
        if self.active_step_elapsed_at(now_ms) < planned_ms {
            return false;
        }

        self.close_current_interval(now_ms);
        if let Some(step) = self.current_step_mut() {
            step.actual_ms = step.planned_ms;
            step.status = StepStatus::Completed;
        }

        let focus_target_reached = self.total_focus_elapsed_at(now_ms)
            >= self
                .cycle_config
                .as_ref()
                .map(|config| config.total_focus_target_ms)
                .unwrap_or(planned_ms);

        if let Some(session) = &mut self.session {
            if focus_target_reached {
                session.status = TimerStatus::Completed;
                session.ended_at_ms = Some(now_ms);
            } else {
                session.status = TimerStatus::AwaitingContinue;
            }
        }

        true
    }

    fn reset_active(&mut self) {
        let now_ms = self.clock.now_ms();
        self.close_current_interval(now_ms);
        self.session = None;
        self.steps.clear();
        self.active_step_index = None;
        self.current_interval_id = None;
        self.cycle_config = None;
    }

    fn status(&self) -> TimerStatus {
        self.session
            .as_ref()
            .map(|session| session.status.clone())
            .unwrap_or(TimerStatus::Idle)
    }

    fn current_step_ref(&self) -> Option<&SessionStep> {
        self.active_step_index
            .and_then(|index| self.steps.get(index))
    }

    fn current_step_mut(&mut self) -> Option<&mut SessionStep> {
        self.active_step_index
            .and_then(|index| self.steps.get_mut(index))
    }

    fn open_interval(&mut self, now_ms: i64, session_id: String, step_id: String, kind: StepKind) {
        let interval_id = Uuid::new_v4().to_string();
        self.intervals.push(TimeInterval {
            id: interval_id.clone(),
            session_id,
            step_id,
            kind,
            started_at_ms: now_ms,
            ended_at_ms: None,
            elapsed_ms: 0,
        });
        self.current_interval_id = Some(interval_id);
    }

    fn close_current_interval(&mut self, now_ms: i64) {
        let Some(interval_id) = self.current_interval_id.take() else {
            return;
        };

        let Some(interval) = self
            .intervals
            .iter_mut()
            .find(|interval| interval.id == interval_id && interval.ended_at_ms.is_none())
        else {
            return;
        };

        interval.ended_at_ms = Some(now_ms);
        interval.elapsed_ms = (now_ms - interval.started_at_ms).max(0);
        let elapsed_ms = interval.elapsed_ms;
        if let Some(step) = self.current_step_mut() {
            step.actual_ms = (step.actual_ms + elapsed_ms).min(step.planned_ms);
        }
    }

    fn active_step_elapsed_at(&self, now_ms: i64) -> i64 {
        let Some(step) = self.current_step_ref() else {
            return 0;
        };
        let open_elapsed = self
            .current_interval_id
            .as_ref()
            .and_then(|id| self.intervals.iter().find(|interval| &interval.id == id))
            .filter(|interval| interval.step_id == step.id)
            .map(|interval| (now_ms - interval.started_at_ms).max(0))
            .unwrap_or(0);

        (step.actual_ms + open_elapsed).min(step.planned_ms)
    }

    fn total_focus_elapsed_at(&self, now_ms: i64) -> i64 {
        self.steps
            .iter()
            .map(|step| {
                if !matches!(step.kind, StepKind::Focus) {
                    return 0;
                }
                let open_elapsed = self
                    .current_interval_id
                    .as_ref()
                    .and_then(|id| self.intervals.iter().find(|interval| &interval.id == id))
                    .filter(|interval| interval.step_id == step.id)
                    .map(|interval| (now_ms - interval.started_at_ms).max(0))
                    .unwrap_or(0);
                (step.actual_ms + open_elapsed).min(step.planned_ms)
            })
            .sum()
    }

    fn next_step_kind(&self) -> Option<StepKind> {
        if self.total_focus_elapsed_at(self.clock.now_ms())
            >= self
                .cycle_config
                .as_ref()
                .map(|config| config.total_focus_target_ms)
                .unwrap_or_default()
        {
            return None;
        }

        match self.current_step_ref().map(|step| &step.kind) {
            Some(StepKind::Focus) => Some(StepKind::ShortBreak),
            Some(StepKind::ShortBreak) => Some(StepKind::Focus),
            None => None,
        }
    }

    fn snapshot_without_transition(&self) -> TimerSnapshot {
        let now_ms = self.clock.now_ms();
        let status = self.status();
        let active_elapsed_ms = match status {
            TimerStatus::Running => self.active_step_elapsed_at(now_ms),
            _ => self
                .current_step_ref()
                .map(|step| step.actual_ms)
                .unwrap_or(0),
        };
        let planned_ms = self
            .current_step_ref()
            .map(|step| step.planned_ms)
            .unwrap_or(0);
        let total_focus_elapsed_ms = self.total_focus_elapsed_at(now_ms);
        let total_focus_target_ms = self
            .cycle_config
            .as_ref()
            .map(|config| config.total_focus_target_ms)
            .unwrap_or(planned_ms);

        TimerSnapshot {
            session_id: self.session.as_ref().map(|session| session.id.clone()),
            mode: self.session.as_ref().map(|session| session.mode.clone()),
            status: status.clone(),
            current_step: self.current_step_ref().map(snapshot_step),
            next_step: self.next_step_kind().and_then(|kind| {
                self.cycle_config.as_ref().map(|config| TimerSnapshotStep {
                    id: String::new(),
                    kind: kind.clone(),
                    index: self.steps.len() as u32,
                    planned_ms: match kind {
                        StepKind::Focus => config.focus_ms,
                        StepKind::ShortBreak => config.break_ms,
                    },
                    actual_ms: 0,
                })
            }),
            cta: cta_for_status(&status, self.next_step_kind()),
            cycle: self.cycle_config.as_ref().map(|config| TimerSnapshotCycle {
                focus_ms: config.focus_ms,
                break_ms: config.break_ms,
                total_focus_target_ms: config.total_focus_target_ms,
                completed_focus_ms: total_focus_elapsed_ms,
                completed_steps: self
                    .steps
                    .iter()
                    .filter(|step| matches!(step.status, StepStatus::Completed))
                    .count() as u32,
            }),
            started_at_ms: self.session.as_ref().map(|session| session.started_at_ms),
            current_interval_started_at_ms: self
                .current_interval_id
                .as_ref()
                .and_then(|id| self.intervals.iter().find(|interval| &interval.id == id))
                .map(|interval| interval.started_at_ms),
            now_ms,
            remaining_ms: (planned_ms - active_elapsed_ms).max(0),
            total_focus_elapsed_ms,
            total_focus_target_ms,
            active_kind_elapsed_ms: active_elapsed_ms,
        }
    }
}

fn snapshot_step(step: &SessionStep) -> TimerSnapshotStep {
    TimerSnapshotStep {
        id: step.id.clone(),
        kind: step.kind.clone(),
        index: step.index,
        planned_ms: step.planned_ms,
        actual_ms: step.actual_ms,
    }
}

fn cta_for_status(status: &TimerStatus, next_kind: Option<StepKind>) -> TimerSnapshotCta {
    match status {
        TimerStatus::Running => TimerSnapshotCta {
            kind: TimerSnapshotCtaKind::Pause,
            label: "Pause".into(),
        },
        TimerStatus::Paused => TimerSnapshotCta {
            kind: TimerSnapshotCtaKind::Resume,
            label: "Resume".into(),
        },
        TimerStatus::AwaitingContinue => match next_kind {
            Some(StepKind::ShortBreak) => TimerSnapshotCta {
                kind: TimerSnapshotCtaKind::StartBreak,
                label: "Start break".into(),
            },
            Some(StepKind::Focus) => TimerSnapshotCta {
                kind: TimerSnapshotCtaKind::StartFocus,
                label: "Start focus".into(),
            },
            None => TimerSnapshotCta {
                kind: TimerSnapshotCtaKind::Restart,
                label: "Restart".into(),
            },
        },
        TimerStatus::Completed => TimerSnapshotCta {
            kind: TimerSnapshotCtaKind::Restart,
            label: "Restart".into(),
        },
        TimerStatus::Idle => TimerSnapshotCta {
            kind: TimerSnapshotCtaKind::StartCycle,
            label: "Start focus".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    };

    use super::*;

    #[derive(Clone)]
    struct ManualClock {
        now_ms: Arc<AtomicI64>,
    }

    impl ManualClock {
        fn new(now_ms: i64) -> Self {
            Self {
                now_ms: Arc::new(AtomicI64::new(now_ms)),
            }
        }

        fn set(&self, now_ms: i64) {
            self.now_ms.store(now_ms, Ordering::SeqCst);
        }
    }

    impl Clock for ManualClock {
        fn now_ms(&self) -> i64 {
            self.now_ms.load(Ordering::SeqCst)
        }
    }

    fn engine_at(now_ms: i64) -> (TimerEngine<ManualClock>, ManualClock) {
        let clock = ManualClock::new(now_ms);
        (TimerEngine::new(clock.clone()), clock)
    }

    #[test]
    fn starting_free_session_creates_running_snapshot() {
        let (mut engine, _clock) = engine_at(1_000);

        let snapshot = engine.start_free_session(25_000);

        assert_eq!(snapshot.status, TimerStatus::Running);
        assert_eq!(snapshot.mode, Some(TimerMode::Free));
        assert_eq!(snapshot.remaining_ms, 25_000);
        assert_eq!(snapshot.total_focus_target_ms, 25_000);
        assert!(snapshot.session_id.is_some());
        assert!(snapshot.current_interval_started_at_ms.is_some());
    }

    #[test]
    fn focus_step_completion_waits_for_manual_break_transition() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 4_000);

        clock.set(3_000);
        assert!(engine.complete_if_due());
        let snapshot = engine.snapshot();

        assert_eq!(snapshot.status, TimerStatus::AwaitingContinue);
        assert_eq!(snapshot.total_focus_elapsed_ms, 2_000);
        assert_eq!(snapshot.current_interval_started_at_ms, None);
        assert_eq!(snapshot.next_step.unwrap().kind, StepKind::ShortBreak);
        assert_eq!(snapshot.cta.kind, TimerSnapshotCtaKind::StartBreak);
    }

    #[test]
    fn continue_cycle_opens_break_interval_after_manual_cta() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 4_000);
        clock.set(3_000);
        engine.complete_if_due();

        clock.set(5_000);
        let snapshot = engine.continue_cycle();

        assert_eq!(snapshot.status, TimerStatus::Running);
        assert_eq!(snapshot.current_step.unwrap().kind, StepKind::ShortBreak);
        assert_eq!(snapshot.current_interval_started_at_ms, Some(5_000));
        assert_eq!(snapshot.total_focus_elapsed_ms, 2_000);
    }

    #[test]
    fn break_running_time_does_not_count_toward_focus_total() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 4_000);
        clock.set(3_000);
        engine.complete_if_due();
        clock.set(5_000);
        engine.continue_cycle();

        clock.set(5_500);
        let snapshot = engine.snapshot();

        assert_eq!(snapshot.current_step.unwrap().kind, StepKind::ShortBreak);
        assert_eq!(snapshot.active_kind_elapsed_ms, 500);
        assert_eq!(snapshot.total_focus_elapsed_ms, 2_000);
    }

    #[test]
    fn break_completion_waits_for_manual_focus_transition() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 4_000);
        clock.set(3_000);
        engine.complete_if_due();
        clock.set(5_000);
        engine.continue_cycle();

        clock.set(6_000);
        assert!(engine.complete_if_due());
        let snapshot = engine.snapshot();

        assert_eq!(snapshot.status, TimerStatus::AwaitingContinue);
        assert_eq!(snapshot.next_step.unwrap().kind, StepKind::Focus);
        assert_eq!(snapshot.cta.kind, TimerSnapshotCtaKind::StartFocus);
        assert_eq!(snapshot.total_focus_elapsed_ms, 2_000);
    }

    #[test]
    fn total_focus_completion_marks_session_completed_without_break_cta() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 2_000);

        clock.set(3_000);
        assert!(engine.complete_if_due());
        let snapshot = engine.snapshot();

        assert_eq!(snapshot.status, TimerStatus::Completed);
        assert!(snapshot.next_step.is_none());
        assert_eq!(snapshot.remaining_ms, 0);
        assert_eq!(snapshot.total_focus_elapsed_ms, 2_000);
    }

    #[test]
    fn paused_time_does_not_count_toward_focus_time() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(10_000, 5_000, 10_000);

        clock.set(2_000);
        engine.pause();
        clock.set(7_000);
        assert_eq!(engine.snapshot().total_focus_elapsed_ms, 1_000);

        engine.resume();
        clock.set(9_000);
        let snapshot = engine.snapshot();

        assert_eq!(snapshot.total_focus_elapsed_ms, 3_000);
        assert_eq!(snapshot.remaining_ms, 7_000);
    }

    #[test]
    fn reset_during_transition_clears_active_state() {
        let (mut engine, clock) = engine_at(1_000);
        engine.start_focus_break_cycle(2_000, 1_000, 4_000);
        clock.set(3_000);
        engine.complete_if_due();

        let snapshot = engine.reset();

        assert_eq!(snapshot.status, TimerStatus::Idle);
        assert!(engine.active_state().is_none());
        assert!(engine.current_interval_id.is_none());
    }
}
