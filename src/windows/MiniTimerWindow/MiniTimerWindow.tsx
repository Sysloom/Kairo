import { type PointerEvent } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { formatDuration } from '../../features/focus/formatDuration';
import { useTimerSnapshot } from '../../features/focus/useTimerSnapshot';
import './MiniTimerWindow.css';

export function MiniTimerWindow() {
  const snapshot = useTimerSnapshot();

  const startWindowDrag = (event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return;
    }

    void getCurrentWindow().startDragging().catch((reason: unknown) => {
      console.warn('No se pudo arrastrar el mini timer', reason);
    });
  };

  return (
    <main
      className={`mini-timer mini-timer--${snapshot.status}`}
      aria-label={`Mini timer ${formatDuration(snapshot.remainingMs)}`}
      onPointerDown={startWindowDrag}
    >
      <span>{formatDuration(snapshot.remainingMs)}</span>
    </main>
  );
}
