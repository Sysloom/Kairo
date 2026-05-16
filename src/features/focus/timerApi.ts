import { invoke } from '@tauri-apps/api/core';
import type { TimerSnapshot } from './focus.types';

export function getTimerSnapshot(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('get_timer_snapshot');
}

export function startFreeSession(durationMs: number): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('start_free_session', { durationMs });
}

export function startFocusBreakCycle(
  focusMs: number,
  breakMs: number,
  totalFocusTargetMs: number,
): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('start_focus_break_cycle', {
    focusMs,
    breakMs,
    totalFocusTargetMs,
  });
}

export function continueTimerCycle(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('continue_timer_cycle');
}

export function pauseTimer(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('pause_timer');
}

export function resumeTimer(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('resume_timer');
}

export function resetTimer(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('reset_timer');
}

export function getTodayFocusMs(): Promise<number> {
  return invoke<number>('get_today_focus_ms');
}

export function showMainWindow(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('show_main_window');
}

export function hideMainWindow(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('hide_main_window');
}

export function showTimerWindow(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('show_timer_window');
}

export function hideTimerWindow(): Promise<TimerSnapshot> {
  return invoke<TimerSnapshot>('hide_timer_window');
}

export function quitApp(): Promise<void> {
  return invoke<void>('quit_app');
}
