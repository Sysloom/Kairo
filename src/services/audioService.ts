import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getToneOption } from '../features/focus/focusDurationPreference';
import { TIMER_ALARM_DISMISSED_EVENT, TIMER_RESET_EVENT, TIMER_STEP_COMPLETED_EVENT } from './tauriEvents';

const GENERATED_ALARM_DURATION_MS = 4_000;
const AUDIO_ALARM_START_TIMEOUT_MS = 2_000;

type BrowserAudioContext = InstanceType<typeof window.AudioContext>;
type BrowserOscillatorNode = ReturnType<BrowserAudioContext['createOscillator']>;
type BrowserGainNode = ReturnType<BrowserAudioContext['createGain']>;
type BrowserAudioElement = InstanceType<typeof window.Audio>;

let audioContext: BrowserAudioContext | null = null;
let alarmTimeoutId: number | null = null;
let oscillator: BrowserOscillatorNode | null = null;
let gain: BrowserGainNode | null = null;
let audioElement: BrowserAudioElement | null = null;
let audioObjectUrl: string | null = null;
let audioPlaybackToken = 0;

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
    void playAudioFileAlarm(tone.url);
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

async function playAudioFileAlarm(audioUrl: string): Promise<void> {
  const playbackToken = nextAudioPlaybackToken();
  const playbackUrl = await resolveAudioPlaybackUrl(audioUrl);

  if (!isCurrentAudioPlayback(playbackToken)) {
    revokeAudioObjectUrl(playbackUrl.objectUrl);
    return;
  }

  const nextAudioElement = getAudioElement();
  let hasStarted = false;
  let fallbackTriggered = false;
  const fallbackToGeneratedAlarm = (reason: unknown) => {
    if (fallbackTriggered || !isCurrentAudioPlayback(playbackToken)) {
      return;
    }

    fallbackTriggered = true;
    console.warn('No se pudo reproducir la alarma seleccionada. Usando tono suave como respaldo.', reason);
    stopTimerAlarm();
    playGeneratedAlarm('suave');
  };

  nextAudioElement.pause();
  replaceAudioObjectUrl(playbackUrl.objectUrl);
  nextAudioElement.src = playbackUrl.url;
  nextAudioElement.load();
  nextAudioElement.volume = 0.72;
  nextAudioElement.currentTime = 0;
  nextAudioElement.onended = stopTimerAlarm;
  nextAudioElement.onerror = () => fallbackToGeneratedAlarm(nextAudioElement.error ?? 'Unknown audio element error');
  nextAudioElement.onplaying = () => {
    hasStarted = true;
    clearAlarmTimeout();
  };

  alarmTimeoutId = window.setTimeout(() => {
    if (!hasStarted) {
      fallbackToGeneratedAlarm('Timed out waiting for audio playback to start');
    }
  }, AUDIO_ALARM_START_TIMEOUT_MS);

  void nextAudioElement.play().catch((reason: unknown) => {
    fallbackToGeneratedAlarm(reason);
  });
}

export function stopTimerAlarm(): void {
  audioPlaybackToken += 1;
  clearAlarmTimeout();

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
    audioElement.onerror = null;
    audioElement.onplaying = null;
    audioElement.pause();
    audioElement.currentTime = 0;
  }

  replaceAudioObjectUrl(null);
}

async function resolveAudioPlaybackUrl(audioUrl: string): Promise<{ url: string; objectUrl: string | null }> {
  try {
    const response = await window.fetch(audioUrl);

    if (!response.ok) {
      throw new Error(`Failed to load audio asset: ${response.status} ${response.statusText}`);
    }

    const sourceBlob = await response.blob();
    const playableBlob = sourceBlob.type === '' || sourceBlob.type === 'application/octet-stream'
      ? new window.Blob([sourceBlob], { type: 'audio/mpeg' })
      : sourceBlob;
    const objectUrl = window.URL.createObjectURL(playableBlob);

    return { url: objectUrl, objectUrl };
  } catch (error) {
    console.warn('No se pudo preparar la alarma como Blob. Probando la URL directa.', error);
    return { url: audioUrl, objectUrl: null };
  }
}

function nextAudioPlaybackToken(): number {
  audioPlaybackToken += 1;
  return audioPlaybackToken;
}

function isCurrentAudioPlayback(playbackToken: number): boolean {
  return playbackToken === audioPlaybackToken;
}

function clearAlarmTimeout(): void {
  if (alarmTimeoutId !== null) {
    window.clearTimeout(alarmTimeoutId);
    alarmTimeoutId = null;
  }
}

function replaceAudioObjectUrl(nextObjectUrl: string | null): void {
  if (audioObjectUrl !== null && audioObjectUrl !== nextObjectUrl) {
    revokeAudioObjectUrl(audioObjectUrl);
  }

  audioObjectUrl = nextObjectUrl;
}

function revokeAudioObjectUrl(objectUrl: string | null): void {
  if (objectUrl !== null) {
    window.URL.revokeObjectURL(objectUrl);
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
