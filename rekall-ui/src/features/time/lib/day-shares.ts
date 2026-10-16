import type { DaySummaryRow } from './day-summary'

/** A working day: the length at which a day's load bar reaches the cell's full width. */
export const FULL_DAY_SECONDS = 8 * 3600

export interface DayShare {
  /** The project's id, or the task's when the task has no project the catalog still knows. */
  readonly id: string
  readonly seconds: number
}

/** A day's time per project, largest first, so the load bar always starts with what dominated it. */
export function projectShares(rows: readonly DaySummaryRow[]): DayShare[] {
  const totals = new Map<string, number>()
  for (const row of rows) {
    const id = row.projectId ?? row.taskId
    totals.set(id, (totals.get(id) ?? 0) + row.totalSeconds)
  }
  return [...totals.entries()].map(([id, seconds]) => ({ id, seconds })).sort((a, b) => b.seconds - a.seconds)
}

/** How full a day was, from 0 to 1, against {@link FULL_DAY_SECONDS}; anything longer is full. */
export function dayLoad(totalSeconds: number): number {
  return Math.min(1, Math.max(0, totalSeconds) / FULL_DAY_SECONDS)
}

export function totalSeconds(rows: readonly DaySummaryRow[]): number {
  return rows.reduce((sum, row) => sum + row.totalSeconds, 0)
}
