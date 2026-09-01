import { dateKey } from '@/common/calendar/month-grid'
import type { Task, TimeEntry } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

export function projectActivitySeries(
  projectId: ProjectId,
  tasks: readonly Task[],
  entries: readonly TimeEntry[],
  nowMs: number,
  days = 14
): number[] {
  const taskIds = new Set(tasks.filter((t) => t.projectId === projectId).map((t) => t.id))
  const totals = new Map<string, number>()
  for (const entry of entries) {
    if (!taskIds.has(entry.taskId)) continue
    const day = dateKey(new Date(entry.startedAt))
    const end = entry.stoppedAt ? Date.parse(entry.stoppedAt) : nowMs
    totals.set(day, (totals.get(day) ?? 0) + (end - Date.parse(entry.startedAt)) / 1000)
  }

  const today = new Date(nowMs)
  const series: number[] = []
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(today.getFullYear(), today.getMonth(), today.getDate() - i)
    series.push(totals.get(dateKey(d)) ?? 0)
  }
  return series
}

export function hasActivity(series: readonly number[]): boolean {
  return series.some((v) => v > 0)
}
