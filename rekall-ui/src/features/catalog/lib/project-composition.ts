import type { TaskRow } from '@/features/task/model/task-row'

export interface ProjectComposition {
  readonly filed: number
  readonly review: number
  readonly active: number
  readonly todo: number
}

/** Where a project's tasks stand: finished, waiting for a decision, being worked, or not started. */
export function projectComposition(rows: readonly TaskRow[]): ProjectComposition {
  const count = (kinds: readonly TaskRow['kind'][]): number => rows.filter((row) => kinds.includes(row.kind)).length
  return {
    filed: count(['filed']),
    review: count(['review']),
    active: count(['running', 'progress', 'blocked']),
    todo: count(['todo', 'backlog'])
  }
}

export function reviewWaiting(rows: readonly TaskRow[]): number {
  return rows.reduce((sum, row) => sum + row.needs, 0)
}
