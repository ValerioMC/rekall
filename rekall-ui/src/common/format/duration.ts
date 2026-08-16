const MINUTE = 60
const HOUR = 60 * MINUTE

/** `HH:MM:SS`, for the live clock on a session that is still running. */
export function formatClock(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  const h = Math.floor(seconds / HOUR)
  const m = Math.floor((seconds % HOUR) / MINUTE)
  const s = seconds % MINUTE
  return [h, m, s].map((part) => String(part).padStart(2, '0')).join(':')
}

/**
 * The shortest phrase that is still true, for totals and recap rows.
 *
 * Zero reads as zero: nothing has been tracked, and saying so plainly is more honest than
 * "under a minute", which implies a session happened. A session that did happen but rounds
 * under a minute gets that phrase instead, so a closed one never reads as having taken no time.
 */
export function formatDuration(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  if (seconds === 0) return '0m'
  if (seconds < MINUTE) return '< 1m'
  const h = Math.floor(seconds / HOUR)
  const m = Math.floor((seconds % HOUR) / MINUTE)
  if (h === 0) return `${m}m`
  return m === 0 ? `${h}h` : `${h}h ${m}m`
}
