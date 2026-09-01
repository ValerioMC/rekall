/** A week, or a month. The two spans a person actually reports on. */
export type ReportPeriod = 'week' | 'month'

export interface PeriodRange {
  readonly period: ReportPeriod
  /** Local midnight on the first day, inclusive. */
  readonly start: Date
  /** Local midnight on the day after the last, exclusive. */
  readonly end: Date
  /** Every day in the span, local midnight, in order. */
  readonly days: readonly Date[]
  /** What to put above the report: `1 – 7 Sep 2026`, or `September 2026`. */
  readonly label: string
}

/** Monday, the way the calendar already lays a week out. */
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

/**
 * The span containing `anchor`, in local time.
 *
 * Local, and not UTC, for the same reason the calendar buckets by local day: a Monday morning
 * is a Monday morning where the person doing the work is, and a report that moves an evening
 * into the next week because of a timezone is a report nobody can check against their memory.
 */
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
  // The month is named once when the week does not cross one, twice when it does: a week that
  // runs from August into September has to say so, and one that does not should not repeat it.
  const sameMonth = start.getMonth() === last.getMonth()
  const from = start.toLocaleDateString(
    undefined,
    sameMonth ? { day: 'numeric' } : { day: 'numeric', month: 'short' }
  )
  const to = last.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })
  return `${from} – ${to}`
}

/** The anchor for the span `delta` periods away, for the arrows either side of the label. */
export function shiftAnchor(period: ReportPeriod, anchor: Date, delta: number): Date {
  return period === 'week'
    ? new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate() + delta * 7)
    : new Date(anchor.getFullYear(), anchor.getMonth() + delta, 1)
}

export function isWithin(range: PeriodRange, date: Date): boolean {
  return date >= range.start && date < range.end
}
