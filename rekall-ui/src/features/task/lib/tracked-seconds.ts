import type { TimeEntry } from '@/features/time/model/time-entry'

const MS_PER_SECOND = 1000

/** Seconds a task's entries add up to; an entry still running counts up to `nowMs`. */
export function trackedSeconds(entries: readonly TimeEntry[], nowMs: number): number {
  return entries.reduce((sum, entry) => {
    const endMs = entry.stoppedAt === null ? nowMs : Date.parse(entry.stoppedAt)
    return sum + Math.max(0, endMs - Date.parse(entry.startedAt)) / MS_PER_SECOND
  }, 0)
}
