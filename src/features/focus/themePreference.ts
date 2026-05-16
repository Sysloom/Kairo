import { useCallback, useEffect, useState } from 'react';
import { emit, listen } from '@tauri-apps/api/event';

const THEME_STORAGE_KEY = 'focus-tray:theme';
const THEME_CHANGED_EVENT = 'focus-theme-changed';
export const THEME_CHANGED_IPC_EVENT = 'kairo://theme-changed';
const DEFAULT_THEME: ThemePreference = 'light';

export const THEME_OPTIONS = [
  { value: 'light', label: 'Claro' },
  { value: 'dark', label: 'Oscuro' },
] as const;

export type ThemePreference = (typeof THEME_OPTIONS)[number]['value'];

export function useThemePreference() {
  const [theme, setThemeState] = useState(readThemePreference);

  useEffect(() => {
    const syncTheme = () => {
      const nextTheme = readThemePreference();
      setThemeState(nextTheme);
      applyThemePreference(nextTheme);
    };

    const unlistenPromise = listen<ThemePreference>(THEME_CHANGED_IPC_EVENT, (event) => {
      const nextTheme = normalizeThemePreference(event.payload);
      setThemeState(nextTheme);
      applyThemePreference(nextTheme);
    });

    syncTheme();
    window.addEventListener('storage', syncTheme);
    window.addEventListener(THEME_CHANGED_EVENT, syncTheme);

    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
      window.removeEventListener('storage', syncTheme);
      window.removeEventListener(THEME_CHANGED_EVENT, syncTheme);
    };
  }, []);

  const setTheme = useCallback((nextTheme: ThemePreference) => {
    const normalizedTheme = normalizeThemePreference(nextTheme);

    window.localStorage.setItem(THEME_STORAGE_KEY, normalizedTheme);
    setThemeState(normalizedTheme);
    applyThemePreference(normalizedTheme);
    window.dispatchEvent(new window.CustomEvent(THEME_CHANGED_EVENT));
    void emit(THEME_CHANGED_IPC_EVENT, normalizedTheme);
  }, []);

  return { theme, setTheme };
}

function readThemePreference(): ThemePreference {
  if (typeof window === 'undefined') {
    return DEFAULT_THEME;
  }

  return normalizeThemePreference(window.localStorage.getItem(THEME_STORAGE_KEY));
}

function normalizeThemePreference(value: unknown): ThemePreference {
  return THEME_OPTIONS.some((option) => option.value === value) ? value as ThemePreference : DEFAULT_THEME;
}

function applyThemePreference(theme: ThemePreference) {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
}
