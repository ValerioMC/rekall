import { newestFirst } from '@/features/catalog/model/task-code'
import type { WorkKind, WorkLane, WorkLaneId, WorkRow } from '@/features/work/model/work-view'

const LANE_OF_KIND: Readonly<Record<WorkKind, WorkLaneId>> = {
  todo: 'todo',
  backlog: 'todo',
  progress: 'progress',
  running: 'progress',
  blocked: 'progress',
  review: 'review',
  filed: 'filed'
}

export const WORK_LANE_LABEL: Readonly<Record<WorkLaneId, string>> = {
  todo: 'To do',
  progress: 'In progress',
  review: 'Awaiting your review',
  filed: 'Filed'
}

const LANE_ORDER: readonly WorkLaneId[] = ['todo', 'progress', 'review', 'filed']

/**
 * The rows dealt into the board's lanes, newest first. The three live lanes together hold exactly
 * the tasks the ledger's Open tab lists; Filed holds the rest, which the board shows as a count.
 */
export function workLanes(rows: readonly WorkRow[]): WorkLane[] {
  return LANE_ORDER.map((id) => ({
    id,
    label: WORK_LANE_LABEL[id],
    rows: rows.filter((row) => LANE_OF_KIND[row.kind] === id).sort((a, b) => newestFirst(a.task, b.task))
  }))
}
