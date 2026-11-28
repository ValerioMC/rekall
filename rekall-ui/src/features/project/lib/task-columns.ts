import { hasDelivery } from '@/features/project/lib/task-delivery'
import type { ListedTask, OptionalTaskColumn, ProjectTaskSortKey } from '@/features/project/model/project-task-list'

export type TaskColumnId = 'peek' | 'mark' | 'task' | 'title' | 'status' | 'plan' | 'updated' | 'actions' | OptionalTaskColumn

export interface TaskColumn {
  readonly id: TaskColumnId
  readonly label: string
  readonly track: string
  readonly sortKey?: ProjectTaskSortKey
  readonly alignEnd?: boolean
}

interface OptionalColumn extends TaskColumn {
  readonly id: OptionalTaskColumn
  /** The narrowest list, in pixels, that has room for the column. */
  readonly minWidth: number
  readonly hasValue: (row: ListedTask) => boolean
}

const TITLE_TRACK = 'minmax(240px, 1fr)'

const STEP: OptionalColumn = { id: 'step', label: 'Step or summary', track: 'minmax(0, 1.15fr)', minWidth: 1420, hasValue: (row) => row.line.text !== '' }
const TAG: OptionalColumn = { id: 'tag', label: 'Tag', track: '118px', sortKey: 'tag', minWidth: 1220, hasValue: (row) => row.task.tagId !== null }
const DELIVERY: OptionalColumn = { id: 'delivery', label: 'Issue and PR', track: '150px', minWidth: 1820, hasValue: (row) => hasDelivery(row.task) }
const NOTES: OptionalColumn = { id: 'notes', label: 'Notes', track: '52px', sortKey: 'notes', alignEnd: true, minWidth: 1620, hasValue: (row) => row.task.documentCount > 0 }
const TOKENS: OptionalColumn = { id: 'tokens', label: 'Tokens', track: '70px', sortKey: 'tokens', alignEnd: true, minWidth: 1100, hasValue: (row) => row.tokens > 0 }
const TIME: OptionalColumn = { id: 'time', label: 'Time', track: '72px', sortKey: 'time', alignEnd: true, minWidth: 1620, hasValue: (row) => row.trackedSeconds > 0 }

export const OPTIONAL_COLUMNS: readonly OptionalColumn[] = [STEP, TAG, DELIVERY, NOTES, TOKENS, TIME]

const COLUMNS_IN_ORDER: readonly (TaskColumn | OptionalColumn)[] = [
  { id: 'peek', label: '', track: '24px' },
  { id: 'mark', label: '', track: '16px' },
  { id: 'task', label: 'Task', track: '72px', sortKey: 'task' },
  { id: 'title', label: 'Title', track: TITLE_TRACK, sortKey: 'title' },
  STEP,
  TAG,
  { id: 'status', label: 'Status', track: '136px', sortKey: 'status' },
  { id: 'plan', label: 'Plan', track: '150px', sortKey: 'plan' },
  DELIVERY,
  NOTES,
  TOKENS,
  TIME,
  { id: 'updated', label: 'Updated', track: '66px', sortKey: 'updated', alignEnd: true },
  { id: 'actions', label: '', track: '56px' }
]

function isOptional(column: TaskColumn): column is OptionalColumn {
  return 'minWidth' in column
}

export interface ColumnChoice {
  readonly width: number
  readonly hidden: ReadonlySet<OptionalTaskColumn>
  /** Every task of the project: a column none of them fills is left out. */
  readonly rows: readonly ListedTask[]
}

/** The columns a list this wide draws, in order. */
export function columnsFor({ width, hidden, rows }: ColumnChoice): TaskColumn[] {
  return COLUMNS_IN_ORDER.filter(
    (column) => !isOptional(column) || (!hidden.has(column.id) && width >= column.minWidth && rows.some(column.hasValue))
  )
}

/** Why a column that is ticked is not on screen, or null when nothing keeps it off. */
export function columnAbsence(id: OptionalTaskColumn, { width, rows }: ColumnChoice): string | null {
  const column = OPTIONAL_COLUMNS.find((candidate) => candidate.id === id)!
  if (!rows.some(column.hasValue)) return 'nothing to show'
  return width >= column.minWidth ? null : 'needs more width'
}

export function gridTemplate(columns: readonly TaskColumn[]): string {
  return columns.map((column) => column.track).join(' ')
}

/** The narrowest list that can give the preview a pane of its own. */
export const SIDE_PREVIEW_MIN_WIDTH = 1500

/** The preview pane and the gap before it, taken from the width the columns share. */
export const SIDE_PREVIEW_SPAN = 408
