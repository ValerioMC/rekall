import type { DiagramSummary } from '@/model/diagram'
import type { TaskId } from '@/model/branded'

/** Where a task stands with its diagrams: none yet, a session drawing one, or at least one drawn. */
export type DiagramPhase = 'none' | 'generating' | 'ready'

export interface TaskDiagrams {
  readonly phase: DiagramPhase
  /** Newest first. Kept while a new one is generated, so the old one stays openable. */
  readonly diagrams: readonly DiagramSummary[]
}

const NONE: TaskDiagrams = { phase: 'none', diagrams: [] }

/** Diagrams grouped by the task they were drawn for; diagrams tied to no task are left out. */
export function diagramsByTask(summaries: readonly DiagramSummary[]): ReadonlyMap<TaskId, readonly DiagramSummary[]> {
  const grouped = new Map<TaskId, DiagramSummary[]>()
  for (const summary of summaries) {
    if (!summary.taskId) continue
    const known = grouped.get(summary.taskId) ?? []
    known.push(summary)
    grouped.set(summary.taskId, known)
  }
  for (const list of grouped.values()) list.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  return grouped
}

/** Generating wins over ready: a regeneration is in progress even when an earlier diagram exists. */
export function taskDiagrams(
  taskId: TaskId,
  byTask: ReadonlyMap<TaskId, readonly DiagramSummary[]>,
  generating: ReadonlySet<TaskId>
): TaskDiagrams {
  const diagrams = byTask.get(taskId) ?? NONE.diagrams
  if (generating.has(taskId)) return { phase: 'generating', diagrams }
  return diagrams.length > 0 ? { phase: 'ready', diagrams } : NONE
}
