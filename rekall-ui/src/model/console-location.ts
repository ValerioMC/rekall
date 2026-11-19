import type { LocationQuery } from 'vue-router'
import { asDocumentId, asTaskId, type DocumentId, type TaskId } from '@/model/branded'

export type PaneFocus = 'note' | 'wrapup' | 'description' | 'steps' | 'deviation' | 'terminal'

const LOCATION_MARK = 'tasks'
const PANES: readonly PaneFocus[] = ['note', 'wrapup', 'description', 'steps', 'deviation', 'terminal']

/** What the console shows: the task and note in view, the pane in front, and the step whose deviation that pane reads. */
export interface ConsoleLocation {
  readonly taskId: TaskId | null
  readonly documentId: DocumentId | null
  readonly pane: PaneFocus
  readonly deviationStep: number | null
}

/**
 * The query a location is written as. `mode=tasks` is always there, so a bare `/` reads as "no
 * location"; links written when notes were a second mode keep it, and any other value is no location.
 */
export type ConsoleLocationQuery = Readonly<Record<'mode' | 'task' | 'note' | 'pane' | 'deviation', string | undefined>>

export function locationToQuery(location: ConsoleLocation): ConsoleLocationQuery {
  return {
    mode: LOCATION_MARK,
    task: location.taskId ?? undefined,
    note: location.documentId ?? undefined,
    pane: location.pane === 'note' ? undefined : location.pane,
    deviation: location.deviationStep === null ? undefined : String(location.deviationStep)
  }
}

/** `null` when the query carries no location, as on a plain link to the console. */
export function locationFromQuery(query: LocationQuery): ConsoleLocation | null {
  if (single(query.mode) !== LOCATION_MARK) return null
  const task = single(query.task)
  const note = single(query.note)
  const pane = single(query.pane)
  const deviation = Number(single(query.deviation))
  return {
    taskId: task ? asTaskId(task) : null,
    documentId: note ? asDocumentId(note) : null,
    pane: isOneOf(PANES, pane) ? pane : 'note',
    deviationStep: Number.isInteger(deviation) && deviation > 0 ? deviation : null
  }
}

export function sameLocation(left: ConsoleLocation, right: ConsoleLocation): boolean {
  return (
    left.taskId === right.taskId &&
    left.documentId === right.documentId &&
    left.pane === right.pane &&
    left.deviationStep === right.deviationStep
  )
}

function single(value: LocationQuery[string] | undefined): string | null {
  const first = Array.isArray(value) ? value[0] : value
  return typeof first === 'string' && first !== '' ? first : null
}

function isOneOf<T extends string>(allowed: readonly T[], value: string | null): value is T {
  return value !== null && (allowed as readonly string[]).includes(value)
}
