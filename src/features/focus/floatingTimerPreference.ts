export type FloatingTimerVariant = 'compact' | 'analog';

const FLOATING_TIMER_VARIANT_KEY = 'focus-tray:floating-timer-variant';
const DEFAULT_FLOATING_TIMER_VARIANT: FloatingTimerVariant = 'compact';

export function getFloatingTimerVariantPreference(): FloatingTimerVariant {
  if (typeof window === 'undefined') {
    return DEFAULT_FLOATING_TIMER_VARIANT;
  }

  const storedVariant = window.localStorage.getItem(FLOATING_TIMER_VARIANT_KEY);

  return isFloatingTimerVariant(storedVariant) ? storedVariant : DEFAULT_FLOATING_TIMER_VARIANT;
}

export function setFloatingTimerVariantPreference(variant: FloatingTimerVariant): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.setItem(FLOATING_TIMER_VARIANT_KEY, variant);
}

function isFloatingTimerVariant(value: string | null): value is FloatingTimerVariant {
  return value === 'compact' || value === 'analog';
}
