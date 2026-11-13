import type { WorkRow } from '@/features/work/model/work-view'

export interface ProjectComposition {
  readonly filed: number
  readonly review: number
  readonly active: number
  readonly todo: number
}

/** Where a project's tasks stand: finished, waiting for a decision, being worked, or not started. */
export function projectComposition(rows: readonly WorkRow[]): ProjectComposition {
  const count = (kinds: readonly WorkRow['kind'][]): number => rows.filter((row) => kinds.includes(row.kind)).length
  return {
    filed: count(['filed']),
    review: count(['review']),
    active: count(['running', 'progress', 'blocked']),
    todo: count(['todo', 'backlog'])
  }
}

export function reviewWaiting(rows: readonly WorkRow[]): number {
  return rows.reduce((sum, row) => sum + row.needs, 0)
}
