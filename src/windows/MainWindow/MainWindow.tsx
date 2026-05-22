import { useEffect, useMemo, useState, type CSSProperties, type KeyboardEvent } from 'react';
import { TimerControls } from '../../components/TimerControls';
import { listen } from '@tauri-apps/api/event';
import { durationMinutesToMs, MAX_DURATION_MINUTES, MIN_DURATION_MINUTES, TONE_OPTIONS, useFocusPreferences } from '../../features/focus/focusDurationPreference';
import { formatDuration } from '../../features/focus/formatDuration';
import { calculateTimerPlan, formatTotalMinutesInput, parseTotalMinutesInput, totalMinutesToMs } from '../../features/focus/timerPlan';
import { THEME_OPTIONS, useThemePreference, type ThemePreference } from '../../features/focus/themePreference';
import { continueTimerCycle, getTodayFocusMs, hideMainWindow, pauseTimer, resetTimer, resumeTimer, showTimerWindow, startFocusBreakCycle, startFreeSession } from '../../features/focus/timerApi';
import { useTimerSnapshot } from '../../features/focus/useTimerSnapshot';
import { stopTimerAlarm } from '../../services/audioService';
import { SHOW_SETTINGS_EVENT } from '../../services/tauriEvents';
import kairoWordmarkUrl from '../../../logo/wordmark/kairo-wordmark-bold.svg';
import kairoWordmarkAccentUrl from '../../../logo/wordmark/kairo-wordmark-accent.svg';
import './MainWindow.css';

type MainView = 'timer' | 'settings';
type SessionMode = 'free' | 'cyclic';

export function MainWindow() {
  const snapshot = useTimerSnapshot();
  const {
    focusMinutes,
    restMinutes,
    tone,
    cycleTotalMinutes,
    setFocusMinutes,
    setRestMinutes,
    setTone,
    setCycleTotalMinutes,
  } = useFocusPreferences();
  const { theme, setTheme } = useThemePreference();
  const [todayFocusMs, setTodayFocusMs] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<MainView>('timer');
  const [sessionMode, setSessionMode] = useState<SessionMode>('cyclic');
  const [draftTotalDuration, setDraftTotalDuration] = useState(() => formatTotalMinutesInput(cycleTotalMinutes));
  const isCyclicFlow = isCyclicSession(snapshot, sessionMode);
  const completionCallout = getCompletionCallout(snapshot, isCyclicFlow);

  const startLabel = useMemo(
    () => snapshot.status === 'completed' && isCyclicFlow
      ? 'Volver a iniciar este ciclo'
      : sessionMode === 'cyclic'
      ? `Iniciar ciclo de ${focusMinutes}/${restMinutes} min`
      : `Iniciar enfoque libre de ${focusMinutes} min`,
    [focusMinutes, isCyclicFlow, restMinutes, sessionMode, snapshot.status],
  );
  const isBeforeStart = snapshot.status === 'idle' || snapshot.status === 'completed';
  const configuredTotalMs = getConfiguredTotalMs(sessionMode, focusMinutes, cycleTotalMinutes);
  const totalRemainingMs = getTotalRemainingMs(snapshot, configuredTotalMs);
  const progressPercent = getProgressPercent(snapshot.activeKindElapsedMs, snapshot.currentStep?.plannedMs ?? 0, snapshot.status);
  const planPreview = getPlanPreview(snapshot, sessionMode, focusMinutes, restMinutes, cycleTotalMinutes);
  const boardTitle = view === 'timer' ? 'Kairo' : 'Configuración';
  const boardSubtitle = view === 'timer' ? getContextStatus(snapshot) : 'Estadísticas primero, reglas después';

  useEffect(() => {
    const unlistenPromise = listen(SHOW_SETTINGS_EVENT, () => {
      setView('settings');
    });

    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    if (isBeforeStart) {
      setDraftTotalDuration(formatTotalMinutesInput(sessionMode === 'cyclic' ? cycleTotalMinutes : focusMinutes));
    }
  }, [cycleTotalMinutes, focusMinutes, isBeforeStart, sessionMode]);

  useEffect(() => {
    let cancelled = false;

    getTodayFocusMs()
      .then((value) => {
        if (!cancelled) {
          setTodayFocusMs(value);
        }
      })
      .catch((reason: unknown) => {
        console.error('No se pudo cargar el total de foco de hoy', reason);
      });

    return () => {
      cancelled = true;
    };
  }, [snapshot.sessionId, snapshot.status, snapshot.totalFocusElapsedMs]);

  const runAction = (action: () => Promise<unknown>) => {
    setError(null);
    void action().catch((reason: unknown) => {
      const message = reason instanceof Error ? reason.message : String(reason);
      setError(message);
    });
  };

  const startSession = () => {
    stopTimerAlarm();
    runAction(async () => {
      if (sessionMode === 'cyclic') {
        const timerPlan = calculateTimerPlan(cycleTotalMinutes, focusMinutes, restMinutes);

        await startFocusBreakCycle(
          durationMinutesToMs(focusMinutes),
          durationMinutesToMs(restMinutes),
          totalMinutesToMs(timerPlan.totalFocusMinutes),
          totalMinutesToMs(cycleTotalMinutes),
        );
      } else {
        await startFreeSession(durationMinutesToMs(focusMinutes));
      }

      await showTimerWindow();
      await hideMainWindow();
    });
  };

  const continueCycle = () => {
    stopTimerAlarm();
    runAction(async () => {
      await continueTimerCycle();
      await showTimerWindow();
      await hideMainWindow();
    });
  };

  const resumeSession = () => {
    stopTimerAlarm();
    runAction(async () => {
      await resumeTimer();
      await showTimerWindow();
      await hideMainWindow();
    });
  };

  const resetSession = () => {
    stopTimerAlarm();
    runAction(resetTimer);
  };

  const commitTotalDuration = () => {
    const parsedTotalMinutes = parseTotalMinutesInput(draftTotalDuration);

    if (parsedTotalMinutes === null) {
      setDraftTotalDuration(formatTotalMinutesInput(sessionMode === 'cyclic' ? cycleTotalMinutes : focusMinutes));
      return;
    }

    if (sessionMode === 'free') {
      setFocusMinutes(parsedTotalMinutes);
    } else {
      setCycleTotalMinutes(parsedTotalMinutes);
    }

    setDraftTotalDuration(formatTotalMinutesInput(parsedTotalMinutes));
  };

  const handleTotalDurationKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter') {
      (event.currentTarget as unknown as { blur: () => void }).blur();
    }

    if (event.key === 'Escape') {
      setDraftTotalDuration(formatTotalMinutesInput(sessionMode === 'cyclic' ? cycleTotalMinutes : focusMinutes));
      (event.currentTarget as unknown as { blur: () => void }).blur();
    }
  };

  return (
    <main className="main-window shell">
      <section className={`kairo-board kairo-board--${view}`} aria-labelledby="main-title">
        <header className="kairo-header">
          <div>
            <h1 id="main-title">
              {view === 'timer' ? (
                <img
                  className="kairo-header__wordmark"
                  src={theme === 'dark' ? kairoWordmarkAccentUrl : kairoWordmarkUrl}
                  alt={boardTitle}
                />
              ) : boardTitle}
            </h1>
            <p>{boardSubtitle}</p>
          </div>
          {view === 'timer' ? (
            <div className="kairo-total-chip" aria-label="Tiempo total configurado">
              <strong>{formatDuration(totalRemainingMs)}</strong>
              <span>Tiempo total</span>
            </div>
          ) : (
            <button type="button" className="kairo-back-button" onClick={() => setView('timer')}>Timer</button>
          )}
        </header>

        {view === 'timer' ? (
          <section className={`timer-view${completionCallout ? ' timer-view--completed' : ''}`} aria-label="Temporizador principal">
            <div className="hero-card">
              <button
                type="button"
                className="hero-card__floating-timer-button"
                aria-label="Volver al timer flotante"
                onClick={() => runAction(async () => {
                  await showTimerWindow();
                  await hideMainWindow();
                })}
              >
                ↗
              </button>
              <div className="timer-ring-outline" aria-hidden="true" />
              <KairoTickRing progressPercent={progressPercent} />
              <div className="timer-value-group">
                {isBeforeStart ? (
                  <input
                    className="timer-ring__time timer-ring__time-input"
                    aria-label="Tiempo total del timer"
                    type="number"
                    min={1}
                    step={1}
                    value={draftTotalDuration}
                    inputMode="numeric"
                    onChange={(event) => setDraftTotalDuration(event.currentTarget.value)}
                    onBlur={commitTotalDuration}
                    onKeyDown={handleTotalDurationKeyDown}
                  />
                ) : (
                  <span className="timer-ring__time">{formatDuration(snapshot.remainingMs)}</span>
                )}
                <span className="timer-ring__status">{getStatusLabel(snapshot)}</span>
              </div>
            </div>
            {completionCallout ? (
              <div className="completion-chip" aria-live="polite">
                <strong>{completionCallout.title}</strong>
                <span>{completionCallout.body}</span>
              </div>
            ) : (
              <p className="plan-chip">{planPreview}</p>
            )}
            <TimerControls
              status={snapshot.status}
              ctaKind={snapshot.cta.kind}
              ctaLabel={snapshot.cta.label}
              startLabel={startLabel}
              settingsLabel="Abrir configuración"
              onStart={startSession}
              onPause={() => runAction(pauseTimer)}
              onResume={resumeSession}
              onContinue={continueCycle}
              onReset={resetSession}
              onSettings={() => setView('settings')}
            />
            <div className="utility-actions" aria-label="Acciones de ventana">
              <button type="button" onClick={() => runAction(hideMainWindow)}>Ocultar Kairo</button>
            </div>
          </section>
        ) : (
          <section className="settings-view" aria-label="Panel de configuración y estadísticas">
            <section className="stats-section" aria-label="Estadísticas de foco">
              <h2>Estadísticas</h2>
              <div className="stats-grid">
                <StatCard tone="today" label="Hoy" value={formatDuration(todayFocusMs)} />
                <StatCard tone="week" label="Semana" value={formatDuration(todayFocusMs)} />
                <StatCard tone="sessions" label="Sesiones" value={snapshot.sessionId ? '1' : '0'} />
                <StatCard tone="completion" label="Completado" value={`${Math.round(getCompletionRatio(snapshot) * 100)}%`} />
              </div>
            </section>
            <h2 className="config-title">Configuración de tiempos</h2>
            <div className="settings-card config-card">
              <label className="config-row config-row--focus">
                <span>Sesión de enfoque</span>
                <span className="config-value-chip">
                  <input
                    type="number"
                    min={MIN_DURATION_MINUTES}
                    max={MAX_DURATION_MINUTES}
                    value={focusMinutes}
                    onChange={(event) => setFocusMinutes(Number(event.currentTarget.value))}
                  />
                  <span>min</span>
                </span>
              </label>
              <label className="config-row config-row--break">
                <span>Descanso</span>
                <span className="config-value-chip">
                  <input
                    type="number"
                    min={MIN_DURATION_MINUTES}
                    max={MAX_DURATION_MINUTES}
                    value={restMinutes}
                    onChange={(event) => setRestMinutes(Number(event.currentTarget.value))}
                  />
                  <span>min</span>
                </span>
              </label>
              <label className="config-row config-row--sound">
                <span>Alarma</span>
                <span className="config-value-chip config-value-chip--sound">
                  <select value={tone} onChange={(event) => setTone(event.currentTarget.value as typeof tone)}>
                    {TONE_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>{option.label}</option>
                    ))}
                  </select>
                </span>
              </label>
              <label className="config-row config-row--theme">
                <span>Tema</span>
                <span className="config-value-chip config-value-chip--theme">
                  <select value={theme} onChange={(event) => setTheme(event.currentTarget.value as ThemePreference)}>
                    {THEME_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>{option.label}</option>
                    ))}
                  </select>
                </span>
              </label>
            </div>
            <div className="mode-shell">
              <span>Modo</span>
              <div className="mode-picker" role="group" aria-label="Modo de sesión">
                <button
                  type="button"
                  className={sessionMode === 'free' ? 'is-selected' : ''}
                  aria-pressed={sessionMode === 'free'}
                  onClick={() => setSessionMode('free')}
                >Libre</button>
                <button
                  type="button"
                  className={sessionMode === 'cyclic' ? 'is-selected' : ''}
                  aria-pressed={sessionMode === 'cyclic'}
                  onClick={() => setSessionMode('cyclic')}
                >Cíclico</button>
              </div>
            </div>
          </section>
        )}

        {error ? <p className="error" role="alert">{error}</p> : null}
      </section>
    </main>
  );
}

function KairoTickRing({ progressPercent }: { progressPercent: number }) {
  const activeTicks = Math.round((progressPercent / 100) * 72);

  return (
    <div className="tick-ring" aria-hidden="true">
      {Array.from({ length: 72 }, (_, index) => (
        <span
          key={index}
          className={index < activeTicks ? 'is-active' : ''}
          style={{ '--tick-index': index } as CSSProperties}
        />
      ))}
    </div>
  );
}

function StatCard({ label, value, tone }: { label: string; value: string; tone: 'today' | 'week' | 'sessions' | 'completion' }) {
  return (
    <div className={`stat-card stat-card--${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function getContextStatus(snapshot: ReturnType<typeof useTimerSnapshot>): string {
  if (snapshot.status === 'idle') {
    return 'Enfoque libre · listo';
  }

  if (snapshot.status === 'awaiting_continue') {
    return snapshot.nextStep?.kind === 'short_break'
      ? 'Buen foco. Cuando estés listo, iniciá el descanso.'
      : 'Descanso terminado. Cuando quieras, arrancá el próximo foco.';
  }

  if (snapshot.status === 'completed') {
    return snapshot.cycle?.breakMs
      ? 'Terminaste todos tus bloques. ¿Querés volver a iniciar la sesión?'
      : 'Sesión completa · guardada';
  }

  const mode = snapshot.currentStep?.kind === 'short_break' ? 'Descanso' : 'Enfoque libre';
  const status = snapshot.status === 'paused' ? 'pausado' : 'en curso';

  return `${mode} · ${status}`;
}

function getStatusLabel(snapshot: ReturnType<typeof useTimerSnapshot>): string {
  if (snapshot.status === 'running') {
    return snapshot.currentStep?.kind === 'short_break' ? 'Descanso activo' : 'Foco activo';
  }

  if (snapshot.status === 'paused') {
    return 'Pausado';
  }

  if (snapshot.status === 'awaiting_continue') {
    return snapshot.nextStep?.kind === 'short_break' ? 'Sigue: descanso' : 'Sigue: enfoque';
  }

  if (snapshot.status === 'completed') {
    return snapshot.cycle?.breakMs ? 'Ciclo terminado' : 'Sesión completa';
  }

  return 'Listo para enfocar';
}

function getProgressPercent(elapsedMs: number, plannedMs: number, status: string): number {
  if (status === 'completed') {
    return 100;
  }

  if (plannedMs <= 0) {
    return 0;
  }

  return Math.min(100, Math.max(0, (elapsedMs / plannedMs) * 100));
}

function getConfiguredTotalMs(sessionMode: SessionMode, focusMinutes: number, cycleTotalMinutes: number): number {
  const focusMs = durationMinutesToMs(focusMinutes);

  if (sessionMode === 'free') {
    return focusMs;
  }

  return totalMinutesToMs(cycleTotalMinutes);
}

function getTotalRemainingMs(snapshot: ReturnType<typeof useTimerSnapshot>, fallbackConfiguredTotalMs: number): number {
  if (snapshot.status === 'idle') {
    return fallbackConfiguredTotalMs;
  }

  if (snapshot.status === 'completed') {
    return 0;
  }

  const plannedTotalMs = getSnapshotPlannedTotalMs(snapshot) ?? fallbackConfiguredTotalMs;
  const completedElapsedMs = getCompletedStepElapsedMs(snapshot);
  const activeElapsedMs = snapshot.status === 'running' || snapshot.status === 'paused'
    ? snapshot.activeKindElapsedMs
    : 0;

  return Math.max(0, plannedTotalMs - completedElapsedMs - activeElapsedMs);
}

function getSnapshotPlannedTotalMs(snapshot: ReturnType<typeof useTimerSnapshot>): number | null {
  if (!snapshot.cycle) {
    return snapshot.totalFocusTargetMs > 0 ? snapshot.totalFocusTargetMs : null;
  }

  return snapshot.cycle.totalCycleTargetMs;
}

function getCompletedStepElapsedMs(snapshot: ReturnType<typeof useTimerSnapshot>): number {
  const cycle = snapshot.cycle;

  if (!cycle) {
    return snapshot.status === 'awaiting_continue' ? snapshot.totalFocusElapsedMs : 0;
  }

  let elapsedMs = 0;

  for (let stepIndex = 0; stepIndex < cycle.completedSteps; stepIndex += 1) {
    elapsedMs += stepIndex % 2 === 0 ? cycle.focusMs : cycle.breakMs;
  }

  return elapsedMs;
}

function getPlanPreview(
  snapshot: ReturnType<typeof useTimerSnapshot>,
  sessionMode: SessionMode,
  focusMinutes: number,
  restMinutes: number,
  cycleTotalMinutes: number,
): string {
  const totalFocusBlocks = getTotalFocusBlocks(snapshot, sessionMode, focusMinutes, restMinutes, cycleTotalMinutes);
  const totalBreakBlocks = getTotalBreakBlocks(snapshot, sessionMode, focusMinutes, restMinutes, cycleTotalMinutes);
  const completedFocusBlocks = getCompletedFocusBlocks(snapshot, sessionMode);
  const completedBreakBlocks = getCompletedBreakBlocks(snapshot, sessionMode);
  const remainingFocusBlocks = Math.max(0, totalFocusBlocks - completedFocusBlocks);
  const remainingBreakBlocks = Math.max(0, totalBreakBlocks - completedBreakBlocks);
  const breakDurationLabel = formatMinutesLabel(snapshot.cycle?.breakMs ?? durationMinutesToMs(restMinutes));
  const focusLabel = pluralize(remainingFocusBlocks, 'bloque de enfoque', 'bloques de enfoque');

  if (remainingBreakBlocks === 0) {
    return `Quedan ${remainingFocusBlocks} ${focusLabel} · sin descansos pendientes`;
  }

  const breakLabel = pluralize(remainingBreakBlocks, 'descanso', 'descansos');

  return `Quedan ${remainingFocusBlocks} ${focusLabel} · ${remainingBreakBlocks} ${breakLabel} de ${breakDurationLabel}`;
}

function getTotalFocusBlocks(
  snapshot: ReturnType<typeof useTimerSnapshot>,
  sessionMode: SessionMode,
  focusMinutes: number,
  restMinutes: number,
  cycleTotalMinutes: number,
): number {
  if (snapshot.cycle) {
    return calculateTimerPlan(
      snapshot.cycle.totalCycleTargetMs / 60_000,
      snapshot.cycle.focusMs / 60_000,
      snapshot.cycle.breakMs / 60_000,
    ).focusBlocks;
  }

  if (sessionMode === 'cyclic') {
    return calculateTimerPlan(cycleTotalMinutes, focusMinutes, restMinutes).focusBlocks;
  }

  if (snapshot.totalFocusTargetMs > 0) {
    return Math.max(1, Math.ceil(snapshot.totalFocusTargetMs / durationMinutesToMs(focusMinutes)));
  }

  return 1;
}

function getTotalBreakBlocks(
  snapshot: ReturnType<typeof useTimerSnapshot>,
  sessionMode: SessionMode,
  focusMinutes: number,
  restMinutes: number,
  cycleTotalMinutes: number,
): number {
  if (snapshot.cycle) {
    return calculateTimerPlan(
      snapshot.cycle.totalCycleTargetMs / 60_000,
      snapshot.cycle.focusMs / 60_000,
      snapshot.cycle.breakMs / 60_000,
    ).breakBlocks;
  }

  if (sessionMode !== 'cyclic') {
    return 0;
  }

  return calculateTimerPlan(cycleTotalMinutes, focusMinutes, restMinutes).breakBlocks;
}

function getCompletedFocusBlocks(snapshot: ReturnType<typeof useTimerSnapshot>, sessionMode: SessionMode): number {
  if (snapshot.cycle) {
    return Math.ceil(snapshot.cycle.completedSteps / 2);
  }

  return sessionMode === 'free' && snapshot.status === 'completed' ? 1 : 0;
}

function getCompletedBreakBlocks(snapshot: ReturnType<typeof useTimerSnapshot>, sessionMode: SessionMode): number {
  if (snapshot.cycle) {
    return Math.floor(snapshot.cycle.completedSteps / 2);
  }

  return sessionMode === 'cyclic' && snapshot.status === 'completed' ? 1 : 0;
}

function pluralize(count: number, singular: string, plural: string): string {
  return count === 1 ? singular : plural;
}

function formatMinutesLabel(durationMs: number): string {
  const minutes = durationMs / 60_000;
  const value = Number.isInteger(minutes) ? minutes.toString() : minutes.toFixed(1).replace('.', ',');

  return `${value} min`;
}

function getCompletionRatio(snapshot: ReturnType<typeof useTimerSnapshot>): number {
  if (snapshot.totalFocusTargetMs <= 0) {
    return snapshot.status === 'completed' ? 1 : 0;
  }

  return Math.min(1, snapshot.totalFocusElapsedMs / snapshot.totalFocusTargetMs);
}

function isCyclicSession(snapshot: ReturnType<typeof useTimerSnapshot>, sessionMode: SessionMode): boolean {
  return (snapshot.cycle?.breakMs ?? 0) > 0 || (snapshot.status === 'idle' && sessionMode === 'cyclic');
}

function getCompletionCallout(snapshot: ReturnType<typeof useTimerSnapshot>, isCyclicFlow: boolean) {
  if (snapshot.status !== 'completed' || !isCyclicFlow) {
    return null;
  }

  return {
    title: 'Excelente trabajo, cerraste todo el ciclo.',
    body: 'Tu foco de hoy ya quedó guardado. ¿Querés volver a iniciar esta sesión?',
  };
}
