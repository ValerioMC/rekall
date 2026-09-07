import { dateKey } from './month-grid'
import type { Task, TimeEntry } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

export interface DaySummaryRow {
  readonly taskId: TaskId
  readonly taskTitle: string
  readonly anchor: string
  readonly projectId: ProjectId | null
  readonly projectLabel: string
  readonly totalSeconds: number
  readonly isRunning: boolean
}

export function summarizeByDay(
  entries: readonly TimeEntry[],
  tasks: readonly Task[],
  nowMs: number
): Map<string, DaySummaryRow[]> {
  const projectIdByTask = new Map(tasks.map((t) => [t.id, t.projectId]))
  const byDay = new Map<string, Map<TaskId, DaySummaryRow>>()

  for (const entry of entries) {
    const day = dateKey(new Date(entry.startedAt))
    const end = entry.stoppedAt ? Date.parse(entry.stoppedAt) : nowMs
    const seconds = (end - Date.parse(entry.startedAt)) / 1000
    const isRunning = entry.stoppedAt === null

    let dayTasks = byDay.get(day)
    if (!dayTasks) {
      dayTasks = new Map()
      byDay.set(day, dayTasks)
    }

    const existing = dayTasks.get(entry.taskId)
    dayTasks.set(entry.taskId, {
      taskId: entry.taskId,
      taskTitle: entry.taskTitle,
      anchor: entry.anchor,
      projectId: projectIdByTask.get(entry.taskId) ?? null,
      projectLabel: entry.projectLabel,
      totalSeconds: (existing?.totalSeconds ?? 0) + seconds,
      isRunning: (existing?.isRunning ?? false) || isRunning
    })
  }

  const result = new Map<string, DaySummaryRow[]>()
  for (const [day, dayTasks] of byDay) {
    result.set(
      day,
      [...dayTasks.values()].sort((a, b) => b.totalSeconds - a.totalSeconds)
    )
  }
  return result
}
