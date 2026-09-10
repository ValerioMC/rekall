const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

/** Time left until an ISO instant as its two coarsest units ("2d 6h"). Past or unparseable is "now". */
export function formatResetIn(iso: string | null, now: number = Date.now()): string {
  if (!iso) return ''
  const target = Date.parse(iso)
  if (Number.isNaN(target)) return ''

  const remaining = target - now
  if (remaining <= 0) return 'now'
  if (remaining < MINUTE) return '< 1m'

  const days = Math.floor(remaining / DAY)
  const hours = Math.floor((remaining % DAY) / HOUR)
  const minutes = Math.floor((remaining % HOUR) / MINUTE)

  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}
