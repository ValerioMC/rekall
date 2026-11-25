const UNITS = ['B', 'KB', 'MB', 'GB'] as const
const STEP = 1024

export function formatFileSize(bytes: number): string {
  let size = bytes
  let unit = 0
  while (size >= STEP && unit < UNITS.length - 1) {
    size /= STEP
    unit += 1
  }
  const shown = unit === 0 || size >= 10 ? Math.round(size) : Math.round(size * 10) / 10
  return `${shown} ${UNITS[unit]}`
}
