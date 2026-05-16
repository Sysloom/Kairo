import { useCallback, useEffect, useState } from 'react';
import blueCheckChimeUrl from '../../../music-alarm/Blue Check Chime.mp3';
import greenCheckToneV2Url from '../../../music-alarm/Green Check Tone v2.mp3';
import greenCheckToneUrl from '../../../music-alarm/Green Check Tone.mp3';
import pingStudyLoopUrl from '../../../music-alarm/Ping Study Loop.mp3';
import softFocusBloomUrl from '../../../music-alarm/Soft Focus Bloom.mp3';
import softFocusSignalUrl from '../../../music-alarm/Soft Focus Signal.mp3';
import softPingLoopV2Url from '../../../music-alarm/Soft Ping Loop v2.mp3';
import softPingLoopUrl from '../../../music-alarm/Soft Ping Loop.mp3';
import softPulseChimeV2Url from '../../../music-alarm/Soft Pulse Chime v2.mp3';
import softPulseChimeUrl from '../../../music-alarm/Soft Pulse Chime.mp3';

const FOCUS_DURATION_STORAGE_KEY = 'focus-tray:free-focus-duration-minutes';
const REST_DURATION_STORAGE_KEY = 'focus-tray:rest-duration-minutes';
const TONE_STORAGE_KEY = 'focus-tray:tone';
const PREFERENCES_CHANGED_EVENT = 'focus-preferences-changed';

export const DEFAULT_DURATION_MINUTES = 25;
export const DEFAULT_REST_MINUTES = 5;
export const MIN_DURATION_MINUTES = 1;
export const MAX_DURATION_MINUTES = 180;

export const TONE_OPTIONS = [
  { value: 'suave', label: 'Tono suave', kind: 'generated' },
  { value: 'campana', label: 'Campana clara', kind: 'generated' },
  { value: 'soft-focus-bloom', label: 'Soft Focus Bloom', kind: 'audio', url: softFocusBloomUrl },
  { value: 'soft-focus-signal', label: 'Soft Focus Signal', kind: 'audio', url: softFocusSignalUrl },
  { value: 'soft-pulse-chime', label: 'Soft Pulse Chime', kind: 'audio', url: softPulseChimeUrl },
  { value: 'soft-pulse-chime-v2', label: 'Soft Pulse Chime v2', kind: 'audio', url: softPulseChimeV2Url },
  { value: 'soft-ping-loop', label: 'Soft Ping Loop', kind: 'audio', url: softPingLoopUrl },
  { value: 'soft-ping-loop-v2', label: 'Soft Ping Loop v2', kind: 'audio', url: softPingLoopV2Url },
  { value: 'ping-study-loop', label: 'Ping Study Loop', kind: 'audio', url: pingStudyLoopUrl },
  { value: 'blue-check-chime', label: 'Blue Check Chime', kind: 'audio', url: blueCheckChimeUrl },
  { value: 'green-check-tone', label: 'Green Check Tone', kind: 'audio', url: greenCheckToneUrl },
  { value: 'green-check-tone-v2', label: 'Green Check Tone v2', kind: 'audio', url: greenCheckToneV2Url },
  { value: 'silencio', label: 'Sin tono', kind: 'silent' },
] as const;

export type TonePreference = (typeof TONE_OPTIONS)[number]['value'];
export type ToneOption = (typeof TONE_OPTIONS)[number];

export function clampDurationMinutes(value: number): number {
  const requestedMinutes = Number.isFinite(value) ? value : DEFAULT_DURATION_MINUTES;

  return Math.min(Math.max(requestedMinutes, MIN_DURATION_MINUTES), MAX_DURATION_MINUTES);
}

export function durationMinutesToMs(durationMinutes: number): number {
  return clampDurationMinutes(durationMinutes) * 60 * 1000;
}

export function useFocusPreferences() {
  const [preferences, setPreferencesState] = useState(readFocusPreferences);

  useEffect(() => {
    const syncFromStorage = () => {
      setPreferencesState(readFocusPreferences());
    };

    window.addEventListener('storage', syncFromStorage);
    window.addEventListener(PREFERENCES_CHANGED_EVENT, syncFromStorage);

    return () => {
      window.removeEventListener('storage', syncFromStorage);
      window.removeEventListener(PREFERENCES_CHANGED_EVENT, syncFromStorage);
    };
  }, []);

  const updatePreferences = useCallback((nextPreferences: Partial<FocusPreferences>) => {
    const currentPreferences = readFocusPreferences();
    const mergedPreferences: FocusPreferences = {
      focusMinutes: clampDurationMinutes(nextPreferences.focusMinutes ?? currentPreferences.focusMinutes),
      restMinutes: clampDurationMinutes(nextPreferences.restMinutes ?? currentPreferences.restMinutes),
      tone: normalizeTonePreference(nextPreferences.tone ?? currentPreferences.tone),
    };

    writeFocusPreferences(mergedPreferences);
    setPreferencesState(mergedPreferences);
    window.dispatchEvent(new window.CustomEvent(PREFERENCES_CHANGED_EVENT));
  }, []);

  return {
    ...preferences,
    setFocusMinutes: (focusMinutes: number) => updatePreferences({ focusMinutes }),
    setRestMinutes: (restMinutes: number) => updatePreferences({ restMinutes }),
    setTone: (tone: TonePreference) => updatePreferences({ tone }),
  };
}

export function useFocusDurationPreference() {
  const { focusMinutes, setFocusMinutes } = useFocusPreferences();

  return { durationMinutes: focusMinutes, setDurationMinutes: setFocusMinutes };
}

type FocusPreferences = {
  focusMinutes: number;
  restMinutes: number;
  tone: TonePreference;
};

function readDurationMinutes(): number {
  const storedValue = window.localStorage.getItem(FOCUS_DURATION_STORAGE_KEY);

  if (storedValue === null) {
    return DEFAULT_DURATION_MINUTES;
  }

  return clampDurationMinutes(Number(storedValue));
}

function readFocusPreferences(): FocusPreferences {
  return {
    focusMinutes: readDurationMinutes(),
    restMinutes: readStoredDurationMinutes(REST_DURATION_STORAGE_KEY, DEFAULT_REST_MINUTES),
    tone: normalizeTonePreference(window.localStorage.getItem(TONE_STORAGE_KEY)),
  };
}

function writeFocusPreferences(preferences: FocusPreferences) {
  window.localStorage.setItem(FOCUS_DURATION_STORAGE_KEY, String(preferences.focusMinutes));
  window.localStorage.setItem(REST_DURATION_STORAGE_KEY, String(preferences.restMinutes));
  window.localStorage.setItem(TONE_STORAGE_KEY, preferences.tone);
}

function readStoredDurationMinutes(storageKey: string, fallbackMinutes: number): number {
  const storedValue = window.localStorage.getItem(storageKey);

  if (storedValue === null) {
    return fallbackMinutes;
  }

  return clampDurationMinutes(Number(storedValue));
}

export function getToneOption(value: unknown): ToneOption {
  return TONE_OPTIONS.find((option) => option.value === value) ?? TONE_OPTIONS[0];
}

function normalizeTonePreference(value: unknown): TonePreference {
  return TONE_OPTIONS.some((option) => option.value === value) ? value as TonePreference : 'suave';
}
