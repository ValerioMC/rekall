import { APP_LOCALE, formatDate } from '@/common/format/app-locale'
export type ReportPeriod = 'week' | 'month'
export type CalendarPeriod = ReportPeriod | 'year'
type Calendar = 'local' | 'utc'

export interface PeriodRange {
  readonly period: ReportPeriod
  readonly start: Date
  readonly end: Date
  readonly days: readonly Date[]
  readonly label: string
}

function calendarDate(period: CalendarPeriod, anchor: Date, offset: number, calendar: Calendar): Date {
  const utc = calendar === 'utc'
  const year = utc ? anchor.getUTCFullYear() : anchor.getFullYear()
  const month = utc ? anchor.getUTCMonth() : anchor.getMonth()
  const day = utc ? anchor.getUTCDate() : anchor.getDate()
  const monthOffset = period === 'year' ? 12 * offset - month : period === 'month' ? offset : 0
  const dayOfMonth = period === 'week' ? day + offset : 1
  return utc ? new Date(Date.UTC(year, month + monthOffset, dayOfMonth)) : new Date(year, month + monthOffset, dayOfMonth)
}

function startOfPeriod(period: CalendarPeriod, anchor: Date, calendar: Calendar): Date {
  const weekday = calendar === 'utc' ? anchor.getUTCDay() : anchor.getDay()
  const mondayOffset = (weekday + 6) % 7
  return calendarDate(period, anchor, period === 'week' ? -mondayOffset : 0, calendar)
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
  const start = startOfPeriod(period, anchor, 'local')
  const end = shiftAnchor(period, start, 1)
  const days = daysBetween(start, end)

  return { period, start, end, days, label: label(period, start, days[days.length - 1]!) }
}

function label(period: ReportPeriod, start: Date, last: Date): string {
  if (period === 'month') {
    return start.toLocaleDateString(APP_LOCALE, { month: 'long', year: 'numeric' })
  }
  return `${formatDate(start)} – ${formatDate(last)}`
}

export function shiftAnchor(period: ReportPeriod, anchor: Date, delta: number): Date {
  return calendarDate(period, anchor, period === 'week' ? delta * 7 : delta, 'local')
}

export function utcPeriodStart(period: CalendarPeriod, instant: string): string {
  return startOfPeriod(period, new Date(instant), 'utc').toISOString().slice(0, 10)
}

export function shiftUtcPeriod(period: CalendarPeriod, start: string, delta: number): string {
  return calendarDate(period, new Date(start), period === 'week' ? delta * 7 : delta, 'utc').toISOString().slice(0, 10)
}

export function isWithin(range: PeriodRange, date: Date): boolean {
  return date >= range.start && date < range.end
}
