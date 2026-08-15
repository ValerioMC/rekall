const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

/**
 * How long ago, in the shortest phrase that is still true.
 *
 * A wrapup is read to decide whether to trust it, and an ISO timestamp does not answer that
 * question: "written 3 days ago" does, at a glance, without arithmetic. Past the point where
 * the distance stops being meaningful the date itself is better, so a month-old wrapup says
 * its date rather than "31 days ago".
 */
export function relativeTime(iso: string, now: number = Date.now()): string {
  const then = Date.parse(iso)
  if (Number.isNaN(then)) return ''

  const elapsed = now - then
  if (elapsed < MINUTE) return 'just now'
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)} min ago`
  if (elapsed < DAY) {
    const hours = Math.floor(elapsed / HOUR)
    return hours === 1 ? 'an hour ago' : `${hours} hours ago`
  }
  if (elapsed < 30 * DAY) {
    const days = Math.floor(elapsed / DAY)
    return days === 1 ? 'yesterday' : `${days} days ago`
  }
  return new Date(then).toLocaleDateString(undefined, {
    day: 'numeric',
    month: 'short',
    year: 'numeric'
  })
}
