export type TimerPlan = {
  totalMinutes: number;
  focusMinutes: number;
  breakMinutes: number;
  focusBlocks: number;
  fullFocusBlocks: number;
  partialFocusMinutes: number;
  breakBlocks: number;
  fullBreakBlocks: number;
  partialBreakMinutes: number;
  omittedPartialBreakMinutes: number;
  totalFocusMinutes: number;
};

export function calculateTimerPlan(
  totalMinutes: number,
  focusMinutes: number,
  breakMinutes: number,
): TimerPlan {
  if (totalMinutes <= 0) {
    throw new Error('totalMinutes must be greater than 0');
  }

  if (focusMinutes <= 0) {
    throw new Error('focusMinutes must be greater than 0');
  }

  if (breakMinutes < 0) {
    throw new Error('breakMinutes cannot be negative');
  }

  const cycleMinutes = focusMinutes + breakMinutes;

  if (breakMinutes === 0) {
    const focusBlocks = Math.ceil(totalMinutes / focusMinutes);
    const partialFocusMinutes = totalMinutes % focusMinutes;
    const fullFocusBlocks = partialFocusMinutes === 0 ? focusBlocks : focusBlocks - 1;

    return {
      totalMinutes,
      focusMinutes,
      breakMinutes,
      focusBlocks,
      fullFocusBlocks,
      partialFocusMinutes: partialFocusMinutes === 0 ? 0 : partialFocusMinutes,
      breakBlocks: 0,
      fullBreakBlocks: 0,
      partialBreakMinutes: 0,
      omittedPartialBreakMinutes: 0,
      totalFocusMinutes: totalMinutes,
    };
  }

  const fullCycles = Math.floor(totalMinutes / cycleMinutes);
  const remaining = totalMinutes % cycleMinutes;

  let focusBlocks = fullCycles;
  let fullFocusBlocks = fullCycles;
  let partialFocusMinutes = 0;
  let breakBlocks = fullCycles;
  let fullBreakBlocks = fullCycles;
  let partialBreakMinutes = 0;
  let omittedPartialBreakMinutes = 0;

  if (remaining > 0 && remaining <= focusMinutes) {
    focusBlocks += 1;

    if (remaining === focusMinutes) {
      fullFocusBlocks += 1;
    } else {
      partialFocusMinutes = remaining;
    }
  }

  if (remaining > focusMinutes) {
    focusBlocks += 1;
    fullFocusBlocks += 1;
    partialBreakMinutes = remaining - focusMinutes;
    // UI rule: do not count the trailing partial break as a full break block.
    omittedPartialBreakMinutes = partialBreakMinutes;
    breakBlocks = Math.max(0, focusBlocks - 1);
    fullBreakBlocks = breakBlocks;
  }

  return {
    totalMinutes,
    focusMinutes,
    breakMinutes,
    focusBlocks,
    fullFocusBlocks,
    partialFocusMinutes,
    breakBlocks,
    fullBreakBlocks,
    partialBreakMinutes,
    omittedPartialBreakMinutes,
    totalFocusMinutes: fullFocusBlocks * focusMinutes + partialFocusMinutes,
  };
}

export function getDefaultCycleTotalMinutes(focusMinutes: number, breakMinutes: number): number {
  // Default editable total for cyclic mode: 2 focus blocks + 1 break.
  return focusMinutes * 2 + breakMinutes;
}

export function clampCycleTotalMinutes(value: number): number {
  const requestedMinutes = Number.isFinite(value) ? value : 1;
  return Math.max(1, Math.round(requestedMinutes));
}

export function totalMinutesToMs(totalMinutes: number): number {
  return clampCycleTotalMinutes(totalMinutes) * 60_000;
}

export function formatTotalMinutesInput(totalMinutes: number): string {
  return String(clampCycleTotalMinutes(totalMinutes));
}

export function parseTotalMinutesInput(value: string): number | null {
  const normalizedValue = value.trim();

  if (!normalizedValue) {
    return null;
  }

  const minutes = Number(normalizedValue.replace(',', '.'));

  if (!Number.isFinite(minutes) || minutes <= 0) {
    return null;
  }

  return clampCycleTotalMinutes(minutes);
}
