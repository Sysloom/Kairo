export type TimerStatus = 'idle' | 'running' | 'paused' | 'awaiting_continue' | 'completed';
export type TimerMode = 'free';
export type StepKind = 'focus' | 'short_break';

export type TimerCtaKind =
  | 'start_cycle'
  | 'pause'
  | 'resume'
  | 'start_break'
  | 'start_focus'
  | 'restart';

export type TimerSnapshotStep = {
  id: string;
  kind: StepKind;
  index: number;
  plannedMs: number;
  actualMs: number;
};

export type TimerSnapshot = {
  sessionId: string | null;
  mode: TimerMode | null;
  status: TimerStatus;
  currentStep: TimerSnapshotStep | null;
  nextStep: TimerSnapshotStep | null;
  cta: {
    kind: TimerCtaKind;
    label: string;
  };
  cycle: {
    focusMs: number;
    breakMs: number;
    totalFocusTargetMs: number;
    totalCycleTargetMs: number;
    completedFocusMs: number;
    completedSteps: number;
  } | null;
  startedAtMs: number | null;
  currentIntervalStartedAtMs: number | null;
  nowMs: number;
  remainingMs: number;
  totalFocusElapsedMs: number;
  totalFocusTargetMs: number;
  activeKindElapsedMs: number;
};

export const idleSnapshot: TimerSnapshot = {
  sessionId: null,
  mode: null,
  status: 'idle',
  currentStep: null,
  nextStep: null,
  cta: {
    kind: 'start_cycle',
    label: 'Iniciar foco',
  },
  cycle: null,
  startedAtMs: null,
  currentIntervalStartedAtMs: null,
  nowMs: Date.now(),
  remainingMs: 0,
  totalFocusElapsedMs: 0,
  totalFocusTargetMs: 0,
  activeKindElapsedMs: 0,
};
