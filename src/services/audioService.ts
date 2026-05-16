import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getToneOption } from '../features/focus/focusDurationPreference';
import { TIMER_ALARM_DISMISSED_EVENT, TIMER_RESET_EVENT, TIMER_STEP_COMPLETED_EVENT } from './tauriEvents';

const GENERATED_ALARM_DURATION_MS = 4_000;

type BrowserAudioContext = InstanceType<typeof window.AudioContext>;
type BrowserOscillatorNode = ReturnType<BrowserAudioContext['createOscillator']>;
type BrowserGainNode = ReturnType<BrowserAudioContext['createGain']>;
type BrowserAudioElement = InstanceType<typeof window.Audio>;

let audioContext: BrowserAudioContext | null = null;
let alarmTimeoutId: number | null = null;
let oscillator: BrowserOscillatorNode | null = null;
let gain: BrowserGainNode | null = null;
let audioElement: BrowserAudioElement | null = null;

export function bindTimerAlarmEvents(): () => void {
  let disposed = false;
  const ownsAlarmPlayback = shouldOwnAlarmPlayback();
  const notifyAlarmDismissed = () => {
    if (ownsAlarmPlayback) {
      stopTimerAlarm();
      return;
    }

    void emit(TIMER_ALARM_DISMISSED_EVENT);
  };
  const unlistenStepCompletedPromise = ownsAlarmPlayback ? listen(TIMER_STEP_COMPLETED_EVENT, () => {
    if (!disposed) {
      playTimerAlarm();
    }
  }) : Promise.resolve(() => undefined);
  const unlistenResetPromise = ownsAlarmPlayback ? listen(TIMER_RESET_EVENT, () => {
    stopTimerAlarm();
  }) : Promise.resolve(() => undefined);
  const unlistenAlarmDismissedPromise = ownsAlarmPlayback ? listen(TIMER_ALARM_DISMISSED_EVENT, () => {
    stopTimerAlarm();
  }) : Promise.resolve(() => undefined);

  window.addEventListener('pointerdown', notifyAlarmDismissed, { capture: true });

  return () => {
    disposed = true;
    window.removeEventListener('pointerdown', notifyAlarmDismissed, { capture: true });
    stopTimerAlarm();
    void unlistenStepCompletedPromise.then((unlisten) => unlisten());
    void unlistenResetPromise.then((unlisten) => unlisten());
    void unlistenAlarmDismissedPromise.then((unlisten) => unlisten());
  };
}

export function playTimerAlarm(): void {
  const tone = getToneOption(readTonePreference());

  if (tone.kind === 'silent') {
    return;
  }

  stopTimerAlarm();

  if (tone.kind === 'audio') {
    playAudioFileAlarm(tone.url);
    return;
  }

  playGeneratedAlarm(tone.value);
}

function playGeneratedAlarm(toneValue: string): void {
  const isClearBell = toneValue === 'campana';

  const context = getAudioContext();
  const nextOscillator = context.createOscillator();
  const nextGain = context.createGain();
  const now = context.currentTime;

  nextOscillator.type = isClearBell ? 'triangle' : 'sine';
  nextOscillator.frequency.setValueAtTime(isClearBell ? 880 : 660, now);
  nextGain.gain.setValueAtTime(0.0001, now);
  nextGain.gain.exponentialRampToValueAtTime(0.18, now + 0.03);
  nextGain.gain.exponentialRampToValueAtTime(0.0001, now + GENERATED_ALARM_DURATION_MS / 1000);

  nextOscillator.connect(nextGain);
  nextGain.connect(context.destination);
  nextOscillator.start(now);
  nextOscillator.stop(now + GENERATED_ALARM_DURATION_MS / 1000);
  nextOscillator.addEventListener('ended', stopTimerAlarm, { once: true });

  oscillator = nextOscillator;
  gain = nextGain;
  alarmTimeoutId = window.setTimeout(stopTimerAlarm, GENERATED_ALARM_DURATION_MS + 100);
}

function playAudioFileAlarm(audioUrl: string): void {
  const nextAudioElement = getAudioElement();

  nextAudioElement.pause();
  nextAudioElement.src = audioUrl;
  nextAudioElement.load();
  nextAudioElement.volume = 0.72;
  nextAudioElement.currentTime = 0;
  nextAudioElement.onended = stopTimerAlarm;

  void nextAudioElement.play().catch((reason: unknown) => {
    console.warn('No se pudo reproducir la alarma seleccionada', reason);
    stopTimerAlarm();
  });
}

export function stopTimerAlarm(): void {
  if (alarmTimeoutId !== null) {
    window.clearTimeout(alarmTimeoutId);
    alarmTimeoutId = null;
  }

  if (oscillator !== null) {
    try {
      oscillator.stop();
    } catch {
      // Already stopped naturally.
    }
    oscillator.disconnect();
    oscillator = null;
  }

  gain?.disconnect();
  gain = null;

  if (audioElement !== null) {
    audioElement.onended = null;
    audioElement.pause();
    audioElement.currentTime = 0;
  }
}

function getAudioElement(): BrowserAudioElement {
  audioElement ??= new window.Audio();
  return audioElement;
}

function getAudioContext(): BrowserAudioContext {
  audioContext ??= new window.AudioContext();

  if (audioContext.state === 'suspended') {
    void audioContext.resume();
  }

  return audioContext;
}

function shouldOwnAlarmPlayback(): boolean {
  try {
    return getCurrentWindow().label === 'main';
  } catch {
    return true;
  }
}

function readTonePreference(): string {
  return window.localStorage.getItem('focus-tray:tone') ?? 'suave';
}
