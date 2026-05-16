import { useCallback, useEffect, useMemo, useState, type CSSProperties, type PointerEvent } from 'react';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TimerControls } from '../../components/TimerControls';
import { getFloatingTimerVariantPreference, setFloatingTimerVariantPreference, type FloatingTimerVariant } from '../../features/focus/floatingTimerPreference';
import { durationMinutesToMs, useFocusPreferences } from '../../features/focus/focusDurationPreference';
import { continueTimerCycle, hideTimerWindow, pauseTimer, resetTimer, resumeTimer, showMainWindow, startFocusBreakCycle, startFreeSession } from '../../features/focus/timerApi';
import { formatDuration } from '../../features/focus/formatDuration';
import type { TimerSnapshot } from '../../features/focus/focus.types';
import { useTimerSnapshot } from '../../features/focus/useTimerSnapshot';
import { stopTimerAlarm } from '../../services/audioService';
import './TimerWindow.css';

type SessionMode = 'free' | 'cyclic';
type TimerWindowSize = { width: number; height: number };

const MIN_TIMER_WINDOW_SCALE = 0.68;

const TIMER_WINDOW_BOUNDS = {
  compact: {
    base: { width: 292, height: 242 },
    min: { width: 199, height: 165 },
    max: { width: 438, height: 363 },
  },
  analog: {
    base: { width: 340, height: 406 },
    min: { width: 231, height: 276 },
    max: { width: 510, height: 609 },
  },
} as const;
const ANALOG_TICK_COUNT = 32;
const WINDOW_DRAG_BLOCK_SELECTOR = 'button, input, select, textarea, a, label, [role="button"], [data-no-window-drag="true"]';

export function TimerWindow() {
  const snapshot = useTimerSnapshot();
  const { focusMinutes, restMinutes } = useFocusPreferences();
  const [sessionMode] = useState<SessionMode>('cyclic');
  const [viewVariant, setViewVariant] = useState<FloatingTimerVariant>(() => getFloatingTimerVariantPreference());
  const [logicalWindowSize, setLogicalWindowSize] = useState<TimerWindowSize>(TIMER_WINDOW_BOUNDS.compact.base);

  const stepTitle = getStepTitle(snapshot, focusMinutes);
  const nextLabel = getNextLabel(snapshot, restMinutes, focusMinutes);
  const activeProgress = getActiveProgress(snapshot);
  const analogMinutes = getDisplayMinutes(snapshot.remainingMs, focusMinutes);
  const windowBounds = TIMER_WINDOW_BOUNDS[viewVariant];
  const baseWidth = windowBounds.base.width;
  const baseHeight = windowBounds.base.height;
  const maxWidth = windowBounds.max.width;
  const maxHeight = windowBounds.max.height;
  const minWidth = windowBounds.min.width;
  const minHeight = windowBounds.min.height;
  const windowScale = getWindowScale(logicalWindowSize, windowBounds.base);

  useEffect(() => {
    const timerWindow = getCurrentWindow();

    const applyWindowBounds = async () => {
      await timerWindow.setResizable(true);
      await timerWindow.setMinSize(new LogicalSize(minWidth, minHeight));
      await timerWindow.setMaxSize(new LogicalSize(maxWidth, maxHeight));
      await timerWindow.setSize(new LogicalSize(baseWidth, baseHeight));
      setLogicalWindowSize({ width: baseWidth, height: baseHeight });
    };

    void applyWindowBounds().catch((reason: unknown) => {
      console.warn('No se pudo ajustar el tamaño del timer flotante', reason);
    });
  }, [baseHeight, baseWidth, maxHeight, maxWidth, minHeight, minWidth]);

  useEffect(() => {
    const timerWindow = getCurrentWindow();

    const readLogicalWindowSize = async () => {
      const [innerSize, scaleFactor] = await Promise.all([
        timerWindow.innerSize(),
        timerWindow.scaleFactor(),
      ]);

      setLogicalWindowSize({
        width: innerSize.width / scaleFactor,
        height: innerSize.height / scaleFactor,
      });
    };

    void readLogicalWindowSize().catch((reason: unknown) => {
      console.warn('No se pudo leer el tamaño del timer flotante', reason);
    });

    let removeResizeListener: (() => void) | undefined;

    void timerWindow.onResized(async ({ payload }) => {
      const scaleFactor = await timerWindow.scaleFactor();

      setLogicalWindowSize({
        width: payload.width / scaleFactor,
        height: payload.height / scaleFactor,
      });
    }).then((unlisten) => {
      removeResizeListener = unlisten;
    }).catch((reason: unknown) => {
      console.warn('No se pudo escuchar el resize del timer flotante', reason);
    });

    return () => {
      removeResizeListener?.();
    };
  }, []);

  const analogTicks = useMemo(
    () => Array.from({ length: ANALOG_TICK_COUNT }, (_, index) => ({
      index,
      isActive: index < Math.round(activeProgress * ANALOG_TICK_COUNT),
    })),
    [activeProgress],
  );

  const startCycle = () => {
    stopTimerAlarm();
    void (sessionMode === 'cyclic'
      ? startFocusBreakCycle(
        durationMinutesToMs(focusMinutes),
        durationMinutesToMs(restMinutes),
        durationMinutesToMs(focusMinutes) * 2,
      )
      : startFreeSession(durationMinutesToMs(focusMinutes)));
  };

  const continueCycle = () => {
    stopTimerAlarm();
    void continueTimerCycle();
  };

  const resetCycle = () => {
    stopTimerAlarm();
    void resetTimer();
  };

  const toggleVariant = () => {
    setViewVariant((current) => {
      const nextVariant = current === 'compact' ? 'analog' : 'compact';
      setFloatingTimerVariantPreference(nextVariant);
      return nextVariant;
    });
  };

  const openMainPanel = () => {
    void showMainWindow()
      .then(() => hideTimerWindow())
      .catch((reason: unknown) => {
        console.warn('No se pudo abrir el panel principal de Kairo', reason);
      });
  };

  const hideFloatingTimer = () => {
    void hideTimerWindow().catch((reason: unknown) => {
      console.warn('No se pudo ocultar el timer flotante de Kairo', reason);
    });
  };

  const startWindowResize = useCallback((event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    void getCurrentWindow().startResizeDragging('SouthEast').catch((reason: unknown) => {
      console.warn('No se pudo iniciar el redimensionado del timer flotante', reason);
    });
  }, []);

  const startWindowDrag = (event: PointerEvent<HTMLElement>) => {
    const target = event.target as HTMLElement | null;
    const dragStartedFromInteractiveElement = Boolean(target?.closest?.(WINDOW_DRAG_BLOCK_SELECTOR));

    if (event.button !== 0 || dragStartedFromInteractiveElement) {
      return;
    }

    void getCurrentWindow().startDragging().catch((reason: unknown) => {
      console.warn('No se pudo iniciar el arrastre del timer flotante', reason);
    });
  };

  return (
    <main
      className={`timer-window timer-window--${viewVariant}`}
      aria-label="Timer flotante de foco"
      onPointerDown={startWindowDrag}
      style={{ '--timer-window-scale': windowScale } as CSSProperties}
    >
      <div className="timer-window__surface">
        <header className="timer-window__header">
          <span>{stepTitle}</span>
          <button
            type="button"
            className="timer-window__variant-button"
            aria-label={viewVariant === 'compact' ? 'Cambiar a vista analógica' : 'Volver a vista compacta'}
            onClick={toggleVariant}
          >
            ↗
          </button>
          <button
            type="button"
            className="timer-window__hide-button"
            aria-label="Ocultar timer flotante"
            onClick={hideFloatingTimer}
          >
            ×
          </button>
        </header>

        {viewVariant === 'analog' ? (
          <section className="timer-window__analog" aria-label="Estado analógico del temporizador">
            <div className="timer-window__ring-outline" aria-hidden="true" />
            <div className="timer-window__tick-ring" aria-hidden="true">
              {analogTicks.map((tick) => (
                <span
                  key={tick.index}
                  className={tick.isActive ? 'is-active' : ''}
                  style={{ '--tick-index': tick.index } as CSSProperties}
                />
              ))}
            </div>
            <div className="timer-window__analog-center" aria-label={`${analogMinutes} minutos restantes`}>
              <span className="timer-window__analog-number">{analogMinutes}</span>
              <span className="timer-window__analog-unit">min</span>
            </div>
            <p className="timer-window__analog-next">{nextLabel}</p>
          </section>
        ) : (
          <section className="timer-window__time-group" aria-label="Estado del temporizador">
            <span className="timer-window__time">{formatDuration(snapshot.remainingMs)}</span>
            <span className="timer-window__next">{nextLabel}</span>
          </section>
        )}

        <div className="timer-window__controls" data-no-window-drag="true">
          <TimerControls
            status={snapshot.status}
            ctaKind={snapshot.cta.kind}
            ctaLabel={snapshot.cta.label}
            startLabel={sessionMode === 'cyclic' ? `Iniciar ${focusMinutes}/${restMinutes}` : `Iniciar ${focusMinutes} min`}
            settingsLabel="Abrir panel principal de Kairo"
            onStart={startCycle}
            onPause={() => void pauseTimer()}
            onResume={() => void resumeTimer()}
            onContinue={continueCycle}
            onReset={resetCycle}
            onSettings={openMainPanel}
          />
        </div>
        <button
          type="button"
          className="timer-window__resize-handle"
          aria-label="Redimensionar timer flotante"
          data-no-window-drag="true"
          onPointerDown={startWindowResize}
        >
          <span />
          <span />
        </button>
      </div>
    </main>
  );
}

function getWindowScale(windowSize: { width: number; height: number }, baseSize: { width: number; height: number }): number {
  return Math.max(MIN_TIMER_WINDOW_SCALE, Math.min(windowSize.width / baseSize.width, windowSize.height / baseSize.height));
}

function getStepTitle(snapshot: TimerSnapshot, fallbackFocusMinutes: number): string {
  if (snapshot.currentStep) {
    const label = snapshot.currentStep.kind === 'short_break' ? 'Descanso' : 'Enfoque';
    const total = snapshot.cycle ? Math.max(1, Math.round(snapshot.cycle.totalFocusTargetMs / Math.max(snapshot.cycle.focusMs, 1))) : 1;
    const position = snapshot.currentStep.kind === 'short_break'
      ? Math.max(1, Math.ceil(snapshot.currentStep.index / 2))
      : Math.floor(snapshot.currentStep.index / 2) + 1;

    return `${label} (${position} de ${total})`;
  }

  if (snapshot.nextStep) {
    return snapshot.nextStep.kind === 'short_break' ? 'Sigue: descanso' : 'Sigue: enfoque';
  }

  return `Enfoque ${fallbackFocusMinutes} min`;
}

function getNextLabel(snapshot: TimerSnapshot, restMinutes: number, focusMinutes: number): string {
  if (snapshot.status === 'awaiting_continue' && snapshot.nextStep) {
    const label = snapshot.nextStep.kind === 'short_break' ? 'descanso' : 'enfoque';
    return `Sigue: ${label} ${formatMinutes(snapshot.nextStep.plannedMs)}`;
  }

  if (snapshot.currentStep?.kind === 'focus') {
    return `Sigue: descanso ${restMinutes} min`;
  }

  if (snapshot.currentStep?.kind === 'short_break') {
    return `Sigue: enfoque ${focusMinutes} min`;
  }

  if (snapshot.status === 'completed') {
    return 'Ciclo guardado en tus estadísticas';
  }

  return `Listo: foco ${focusMinutes} min · descanso ${restMinutes} min`;
}

function getActiveProgress(snapshot: TimerSnapshot): number {
  const plannedMs = snapshot.currentStep?.plannedMs ?? 0;

  if (plannedMs <= 0) {
    return 0;
  }

  return Math.min(1, Math.max(0, snapshot.activeKindElapsedMs / plannedMs));
}

function getDisplayMinutes(remainingMs: number, fallbackFocusMinutes: number): number {
  if (remainingMs <= 0) {
    return fallbackFocusMinutes;
  }

  return Math.max(1, Math.ceil(remainingMs / 60_000));
}

function formatMinutes(ms: number): string {
  return `${Math.max(1, Math.ceil(ms / 60_000))} min`;
}
