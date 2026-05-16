export function formatDuration(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${padTwo(minutes)}:${padTwo(seconds)}`;
  }

  return `${padTwo(minutes)}:${padTwo(seconds)}`;
}

function padTwo(value: number): string {
  return value.toString().padStart(2, '0');
}
