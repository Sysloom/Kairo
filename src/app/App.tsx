import { useEffect } from 'react';
import { useThemePreference } from '../features/focus/themePreference';
import { bindTimerAlarmEvents } from '../services/audioService';
import { WindowRouter } from './window-router';

export function App() {
  useThemePreference();
  useEffect(() => bindTimerAlarmEvents(), []);

  return <WindowRouter />;
}
