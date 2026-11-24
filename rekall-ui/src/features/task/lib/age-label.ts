const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

/** How long ago, in at most three characters: `now`, `12m`, `3h`, `2d`, `5w`, `4mo`, `2y`. */
export function ageLabel(iso: string, now: number): string {
  const elapsed = now - Date.parse(iso)
  if (Number.isNaN(elapsed) || elapsed < MINUTE) return 'now'
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)}m`
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)}h`
  const days = Math.floor(elapsed / DAY)
  if (days < 14) return `${days}d`
  if (days < 60) return `${Math.floor(days / 7)}w`
  if (days < 365) return `${Math.floor(days / 30)}mo`
  return `${Math.floor(days / 365)}y`
}
