import { dateKey } from './month-grid'
import type { TimeEntry } from '@/model/catalog'

export interface TrackedDay {
  readonly key: string
  readonly date: Date
  readonly seconds: number
  readonly isToday: boolean
}

/**
 * Seconds tracked on each of the last `days` calendar days, oldest first and today last. An entry
 * counts on the day it started, as the calendar files it; a running one runs up to `nowMs`.
 */
export function trackedLastDays(
  entries: readonly TimeEntry[],
  nowMs: number,
  days = 7
): TrackedDay[] {
  const totals = new Map<string, number>()
  for (const entry of entries) {
    const started = Date.parse(entry.startedAt)
    const ended = entry.stoppedAt ? Date.parse(entry.stoppedAt) : nowMs
    const key = dateKey(new Date(started))
    totals.set(key, (totals.get(key) ?? 0) + Math.max(0, ended - started) / 1000)
  }

  const today = new Date(nowMs)
  return Array.from({ length: days }, (_, index) => {
    const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() - (days - 1 - index))
    const key = dateKey(date)
    return { key, date, seconds: totals.get(key) ?? 0, isToday: index === days - 1 }
  })
}
