import { compareText } from '@/features/catalog/lib/compare-text'
import { isEpic } from '@/features/catalog/model/catalog'
import {
  TASK_FILTER_KEYS,
  type TaskFilterKey,
  type TaskFilterOption,
  type TaskFilters
} from '@/features/project/model/project-task-list'
import { TASK_ROW_KIND_LABEL, type TaskRow, type TaskRowKind } from '@/features/task/model/task-row'

/** The value of a task that has no tag, or belongs to no epic. */
export const NO_VALUE = 'none'

const TYPE_LABEL: Readonly<Record<string, string>> = { planned: 'Planned', quick: 'Quick', epic: 'Epic' }

const FIXED_ORDER: Partial<Readonly<Record<TaskFilterKey, readonly string[]>>> = {
  status: ['running', 'review', 'progress', 'blocked', 'todo', 'backlog', 'filed'],
  type: ['planned', 'quick', 'epic']
}

function typeOf(row: TaskRow): string {
  if (isEpic(row.task)) return 'epic'
  return row.task.quick ? 'quick' : 'planned'
}

const VALUE_OF: Readonly<Record<TaskFilterKey, (row: TaskRow) => string>> = {
  status: (row) => row.kind,
  tag: (row) => row.task.tagId ?? NO_VALUE,
  epic: (row) => (isEpic(row.task) ? row.task.id : (row.task.epicId ?? NO_VALUE)),
  type: typeOf
}

/** Whether the row passes every filter; `ignored` leaves one out, to count what its own options would list. */
export function passesFilters(row: TaskRow, filters: TaskFilters, ignored?: TaskFilterKey): boolean {
  return TASK_FILTER_KEYS.every(
    (key) => key === ignored || filters[key].length === 0 || filters[key].includes(VALUE_OF[key](row))
  )
}

export function hasFilters(filters: TaskFilters): boolean {
  return TASK_FILTER_KEYS.some((key) => filters[key].length > 0)
}

/** A tag or an epic the project no longer has stops narrowing the list: nobody could untick it. */
export function applicableFilters(filters: TaskFilters, rows: readonly TaskRow[]): TaskFilters {
  const present = (key: TaskFilterKey): string[] => {
    const values = new Set(rows.map(VALUE_OF[key]))
    return filters[key].filter((value) => values.has(value))
  }
  return { status: filters.status, tag: present('tag'), epic: present('epic'), type: filters.type }
}

function labelOf(key: TaskFilterKey, value: string, rows: readonly TaskRow[]): string {
  if (key === 'status') return TASK_ROW_KIND_LABEL[value as TaskRowKind]
  if (key === 'type') return TYPE_LABEL[value] ?? value
  if (key === 'tag') return rows.find((row) => row.task.tagId === value)?.task.tagName ?? 'No tag'
  return rows.find((row) => row.task.id === value)?.task.title ?? 'No epic'
}

function inOptionOrder(key: TaskFilterKey): (a: TaskFilterOption, b: TaskFilterOption) => number {
  const order = FIXED_ORDER[key]
  if (order) return (a, b) => order.indexOf(a.value) - order.indexOf(b.value)
  return (a, b) => Number(a.value === NO_VALUE) - Number(b.value === NO_VALUE) || compareText(a.label, b.label)
}

/**
 * What one filter offers: each value some candidate holds, with how many, plus the ones already
 * picked. `rows` is the whole project, which is where the names come from.
 */
export function filterOptions(
  key: TaskFilterKey,
  candidates: readonly TaskRow[],
  picked: readonly string[],
  rows: readonly TaskRow[]
): TaskFilterOption[] {
  const counts = new Map<string, number>(picked.map((value) => [value, 0]))
  for (const row of candidates) {
    const value = VALUE_OF[key](row)
    counts.set(value, (counts.get(value) ?? 0) + 1)
  }
  return [...counts]
    .map(([value, count]) => ({ value, count, label: labelOf(key, value, rows) }))
    .sort(inOptionOrder(key))
}
