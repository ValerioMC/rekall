import type { LocationQuery } from 'vue-router'
import { asDocumentId, asTaskId, type DocumentId, type TaskId } from '@/model/branded'

export type NavMode = 'tasks' | 'notes'

export type PaneFocus = 'note' | 'wrapup' | 'description' | 'steps' | 'terminal'

const NAV_MODES: readonly NavMode[] = ['tasks', 'notes']
const PANES: readonly PaneFocus[] = ['note', 'wrapup', 'description', 'steps', 'terminal']

/** What the console shows: the side being browsed, the task and note in view, the pane in front. */
export interface ConsoleLocation {
  readonly mode: NavMode
  readonly taskId: TaskId | null
  readonly documentId: DocumentId | null
  readonly pane: PaneFocus
}

/** The query a location is written as. `mode` is always there, so a bare `/` reads as "no location". */
export type ConsoleLocationQuery = Readonly<Record<'mode' | 'task' | 'note' | 'pane', string | undefined>>

export function locationToQuery(location: ConsoleLocation): ConsoleLocationQuery {
  return {
    mode: location.mode,
    task: location.taskId ?? undefined,
    note: location.documentId ?? undefined,
    pane: location.pane === 'note' ? undefined : location.pane
  }
}

/** `null` when the query carries no location, as on a plain link to the console. */
export function locationFromQuery(query: LocationQuery): ConsoleLocation | null {
  const mode = single(query.mode)
  if (!isOneOf(NAV_MODES, mode)) return null
  const task = single(query.task)
  const note = single(query.note)
  const pane = single(query.pane)
  return {
    mode,
    taskId: task ? asTaskId(task) : null,
    documentId: note ? asDocumentId(note) : null,
    pane: isOneOf(PANES, pane) ? pane : 'note'
  }
}

export function sameLocation(left: ConsoleLocation, right: ConsoleLocation): boolean {
  return (
    left.mode === right.mode &&
    left.taskId === right.taskId &&
    left.documentId === right.documentId &&
    left.pane === right.pane
  )
}

function single(value: LocationQuery[string] | undefined): string | null {
  const first = Array.isArray(value) ? value[0] : value
  return typeof first === 'string' && first !== '' ? first : null
}

function isOneOf<T extends string>(allowed: readonly T[], value: string | null): value is T {
  return value !== null && (allowed as readonly string[]).includes(value)
}
