import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getTimerSnapshot } from './timerApi';
import { idleSnapshot, type TimerSnapshot, type TimerSnapshotStep } from './focus.types';
import { TIMER_SNAPSHOT_EVENT } from '../../services/tauriEvents';

export function useTimerSnapshot(): TimerSnapshot {
  const [snapshot, setSnapshot] = useState<TimerSnapshot>(idleSnapshot);
  const latestSnapshotRef = useRef<TimerSnapshot>({ ...idleSnapshot, nowMs: 0 });
  const [cosmeticNowMs, setCosmeticNowMs] = useState(() => Date.now());

  const acceptSnapshot = useCallback((nextSnapshot: TimerSnapshot) => {
    const currentSnapshot = latestSnapshotRef.current;

    if (!isFreshSnapshot(nextSnapshot, currentSnapshot)) {
      return;
    }

    latestSnapshotRef.current = nextSnapshot;
    setSnapshot(nextSnapshot);
    setCosmeticNowMs(Date.now());
  }, []);

  useEffect(() => {
    let cancelled = false;

    const unlistenPromise = listen<TimerSnapshot>(TIMER_SNAPSHOT_EVENT, (event) => {
      if (!cancelled) {
        acceptSnapshot(event.payload);
      }
    });

    getTimerSnapshot()
      .then((nextSnapshot) => {
        if (!cancelled) {
          acceptSnapshot(nextSnapshot);
        }
      })
      .catch((error: unknown) => {
        console.error('Failed to load timer snapshot', error);
      });

    return () => {
      cancelled = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [acceptSnapshot]);

  useEffect(() => {
    if (snapshot.status !== 'running' || snapshot.currentIntervalStartedAtMs === null) {
      return;
    }

    const intervalId = window.setInterval(() => {
      setCosmeticNowMs(Date.now());
    }, 1_000);

    return () => window.clearInterval(intervalId);
  }, [snapshot.currentIntervalStartedAtMs, snapshot.status]);

  const renderedSnapshot = useMemo(
    () => withCosmeticTick(snapshot, cosmeticNowMs),
    [cosmeticNowMs, snapshot],
  );

  return renderedSnapshot;
}

function isFreshSnapshot(nextSnapshot: TimerSnapshot, currentSnapshot: TimerSnapshot): boolean {
  return nextSnapshot.nowMs >= currentSnapshot.nowMs;
}

function withCosmeticTick(snapshot: TimerSnapshot, cosmeticNowMs: number): TimerSnapshot {
  if (snapshot.status !== 'running' || snapshot.currentIntervalStartedAtMs === null) {
    return snapshot;
  }

  const currentStep = snapshot.currentStep;

  if (currentStep === null) {
    return snapshot;
  }

  const elapsedAtSnapshotMs = Math.max(0, snapshot.nowMs - snapshot.currentIntervalStartedAtMs);
  const liveElapsedMs = Math.max(0, cosmeticNowMs - snapshot.currentIntervalStartedAtMs);
  const activeKindElapsedMs = Math.min(
    currentStep.plannedMs,
    Math.max(0, snapshot.activeKindElapsedMs - elapsedAtSnapshotMs) + liveElapsedMs,
  );
  const totalFocusElapsedMs = getCosmeticTotalFocusElapsedMs(
    snapshot,
    currentStep,
    elapsedAtSnapshotMs,
    liveElapsedMs,
  );
  const remainingMs = Math.max(0, currentStep.plannedMs - activeKindElapsedMs);

  return {
    ...snapshot,
    nowMs: cosmeticNowMs,
    remainingMs,
    totalFocusElapsedMs,
    activeKindElapsedMs,
    currentStep: {
      ...currentStep,
      actualMs: activeKindElapsedMs,
    },
  };
}

function getCosmeticTotalFocusElapsedMs(
  snapshot: TimerSnapshot,
  currentStep: TimerSnapshotStep,
  elapsedAtSnapshotMs: number,
  liveElapsedMs: number,
): number {
  if (currentStep.kind !== 'focus') {
    return snapshot.totalFocusElapsedMs;
  }

  const closedFocusElapsedMs = Math.max(0, snapshot.totalFocusElapsedMs - elapsedAtSnapshotMs);

  return Math.min(snapshot.totalFocusTargetMs, closedFocusElapsedMs + liveElapsedMs);
}
