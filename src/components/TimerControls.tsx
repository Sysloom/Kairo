import type { TimerCtaKind, TimerStatus } from '../features/focus/focus.types';
import { IconButton } from './IconButton';

type TimerControlsProps = {
  status: TimerStatus;
  ctaKind: TimerCtaKind;
  startLabel?: string;
  ctaLabel?: string;
  settingsLabel?: string;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onContinue: () => void;
  onReset: () => void;
  onSettings?: () => void;
};

export function TimerControls({
  status,
  ctaKind,
  startLabel = 'Iniciar sesión',
  ctaLabel,
  settingsLabel = 'Abrir configuración',
  onStart,
  onPause,
  onResume,
  onContinue,
  onReset,
  onSettings,
}: TimerControlsProps) {
  const isRunning = status === 'running';
  const isPaused = status === 'paused';
  const isAwaitingContinue = status === 'awaiting_continue';
  const canReset = status !== 'idle' && status !== 'completed';

  const primaryAction = getPrimaryAction({
    ctaKind,
    ctaLabel,
    isAwaitingContinue,
    isPaused,
    isRunning,
    onContinue,
    onPause,
    onResume,
    onStart,
    startLabel,
  });

  return (
    <div className="timer-controls" aria-label="Controles del temporizador">
      <IconButton
        className="timer-controls__reset"
        label="Reiniciar sesión"
        icon={<ResetIcon />}
        disabled={!canReset}
        onClick={onReset}
      />
      <IconButton
        className="timer-controls__primary"
        label={primaryAction.label}
        icon={primaryAction.icon}
        onClick={primaryAction.onClick}
      />
      {onSettings ? (
        <IconButton
          className="timer-controls__settings"
          label={settingsLabel}
          icon={<SettingsIcon />}
          onClick={onSettings}
        />
      ) : null}
    </div>
  );
}

type PrimaryActionInput = {
  ctaKind: TimerCtaKind;
  ctaLabel?: string;
  isAwaitingContinue: boolean;
  isPaused: boolean;
  isRunning: boolean;
  onContinue: () => void;
  onPause: () => void;
  onResume: () => void;
  onStart: () => void;
  startLabel: string;
};

function getPrimaryAction(input: PrimaryActionInput) {
  if (input.isRunning) {
    return { label: 'Pausar', icon: <PauseIcon />, onClick: input.onPause };
  }

  if (input.isPaused) {
    return { label: 'Reanudar', icon: <PlayIcon />, onClick: input.onResume };
  }

  if (input.isAwaitingContinue) {
    return {
      label: getContinueLabel(input.ctaKind, input.ctaLabel),
      icon: <PlayIcon />,
      onClick: input.onContinue,
    };
  }

  return { label: input.startLabel, icon: <PlayIcon />, onClick: input.onStart };
}

function getContinueLabel(ctaKind: TimerCtaKind, ctaLabel?: string): string {
  if (ctaKind === 'start_break') {
    return 'Iniciar descanso';
  }

  if (ctaKind === 'start_focus') {
    return 'Iniciar foco';
  }

  return ctaLabel ?? 'Continuar';
}

function PlayIcon() {
  return (
    <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
      <path d="M8 5.6v12.8c0 .8.9 1.3 1.6.9l9.6-6.4c.6-.4.6-1.3 0-1.7L9.6 4.7C8.9 4.3 8 4.8 8 5.6Z" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
      <path d="M8 5.5c0-.8.7-1.5 1.5-1.5S11 4.7 11 5.5v13c0 .8-.7 1.5-1.5 1.5S8 19.3 8 18.5v-13Zm5 0c0-.8.7-1.5 1.5-1.5S16 4.7 16 5.5v13c0 .8-.7 1.5-1.5 1.5S13 19.3 13 18.5v-13Z" />
    </svg>
  );
}

function ResetIcon() {
  return (
    <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
      <path d="M12 5a7 7 0 1 1-6.5 9.6 1.2 1.2 0 0 1 2.2-.9A4.6 4.6 0 1 0 8.9 8H7.2c-.7 0-1.2-.5-1.2-1.2V5.1a1.2 1.2 0 1 1 2.4 0v.2A7 7 0 0 1 12 5Z" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      focusable="false"
      aria-hidden="true"
    >
      <path d="M10 5H3" />
      <path d="M12 19H3" />
      <path d="M14 3v4" />
      <path d="M16 17v4" />
      <path d="M21 12h-9" />
      <path d="M21 19h-5" />
      <path d="M21 5h-7" />
      <path d="M8 10v4" />
      <path d="M8 12H3" />
    </svg>
  );
}
