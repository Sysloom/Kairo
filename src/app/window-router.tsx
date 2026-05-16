import { getCurrentWindow } from '@tauri-apps/api/window';
import { MainWindow } from '../windows/MainWindow/MainWindow';
import { MiniTimerWindow } from '../windows/MiniTimerWindow/MiniTimerWindow';
import { TimerWindow } from '../windows/TimerWindow/TimerWindow';

export function WindowRouter() {
  const label = getWindowLabel();

  if (label === 'timer') {
    return <TimerWindow />;
  }

  if (label === 'mini-timer') {
    return <MiniTimerWindow />;
  }

  return <MainWindow />;
}

function getWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return 'main';
  }
}
