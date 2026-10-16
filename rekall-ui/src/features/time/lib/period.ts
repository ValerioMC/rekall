export type ReportPeriod = 'week' | 'month'

export interface PeriodRange {
  readonly period: ReportPeriod
  readonly start: Date
  readonly end: Date
  readonly days: readonly Date[]
  readonly label: string
}

function startOfWeek(date: Date): Date {
  const mondayOffset = (date.getDay() + 6) % 7
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() - mondayOffset)
}

function daysBetween(start: Date, end: Date): Date[] {
  const days: Date[] = []
  for (
    let day = new Date(start);
    day < end;
    day = new Date(day.getFullYear(), day.getMonth(), day.getDate() + 1)
  ) {
    days.push(day)
  }
  return days
}

export function periodRange(period: ReportPeriod, anchor: Date): PeriodRange {
  const start =
    period === 'week'
      ? startOfWeek(anchor)
      : new Date(anchor.getFullYear(), anchor.getMonth(), 1)
  const end =
    period === 'week'
      ? new Date(start.getFullYear(), start.getMonth(), start.getDate() + 7)
      : new Date(start.getFullYear(), start.getMonth() + 1, 1)
  const days = daysBetween(start, end)

  return { period, start, end, days, label: label(period, start, days[days.length - 1]!) }
}

function label(period: ReportPeriod, start: Date, last: Date): string {
  if (period === 'month') {
    return start.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })
  }
  const sameMonth = start.getMonth() === last.getMonth()
  const from = start.toLocaleDateString(
    undefined,
    sameMonth ? { day: 'numeric' } : { day: 'numeric', month: 'short' }
  )
  const to = last.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })
  return `${from} – ${to}`
}

export function shiftAnchor(period: ReportPeriod, anchor: Date, delta: number): Date {
  return period === 'week'
    ? new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate() + delta * 7)
    : new Date(anchor.getFullYear(), anchor.getMonth() + delta, 1)
}

export function isWithin(range: PeriodRange, date: Date): boolean {
  return date >= range.start && date < range.end
}
