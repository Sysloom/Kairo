import { formatDuration } from '../features/focus/formatDuration';
import type { StepKind, TimerStatus } from '../features/focus/focus.types';

type TimerDisplayProps = {
  remainingMs: number;
  status: TimerStatus;
  currentStepKind?: StepKind | null;
  nextStepKind?: StepKind | null;
};

export function TimerDisplay({ remainingMs, status, currentStepKind = null, nextStepKind = null }: TimerDisplayProps) {
  const statusLabel = getTimerStatusLabel(status, currentStepKind, nextStepKind);

  return (
    <div className="timer-display" aria-label={`Estado del temporizador: ${statusLabel}`}>
      <span className="timer-display__time">{formatDuration(remainingMs)}</span>
      <span className="timer-display__status">{statusLabel}</span>
    </div>
  );
}

function getTimerStatusLabel(
  status: TimerStatus,
  currentStepKind: StepKind | null,
  nextStepKind: StepKind | null,
): string {
  switch (status) {
    case 'running':
      return currentStepKind === 'short_break' ? 'Descanso activo' : 'Foco activo';
    case 'paused':
      return 'Pausado';
    case 'awaiting_continue':
      return nextStepKind === 'short_break' ? 'Foco terminado · iniciá descanso' : 'Descanso terminado · iniciá foco';
    case 'completed':
      return 'Ciclo completado';
    case 'idle':
    default:
      return 'Listo para enfocar';
  }
}
