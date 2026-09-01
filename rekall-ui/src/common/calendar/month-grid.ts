/** Monday first, the way the calendar page lays its header row out. */
export const WEEKDAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as const

/**
 * Six full Monday-start weeks (42 days) that between them cover every day of the given month,
 * plus whatever leading and trailing days from the neighbouring months fill out those weeks —
 * the grid a monthly calendar actually draws, not just the days that belong to the month.
 *
 * `month` is zero-indexed, the same convention `Date` itself uses.
 */
export function monthGridDays(year: number, month: number): Date[] {
  const firstOfMonth = new Date(year, month, 1)
  const mondayOffset = (firstOfMonth.getDay() + 6) % 7
  const start = new Date(year, month, 1 - mondayOffset)
  return Array.from(
    { length: 42 },
    (_, i) => new Date(start.getFullYear(), start.getMonth(), start.getDate() + i)
  )
}

export function isSameDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  )
}

export function isSameMonth(date: Date, year: number, month: number): boolean {
  return date.getFullYear() === year && date.getMonth() === month
}

/** `YYYY-MM-DD` in local time, so a day boundary lands where the person looking at it is. */
export function dateKey(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}
