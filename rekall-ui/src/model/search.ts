import type { DocumentId, TaskId, TaskStepId } from '@/model/branded'

/** Which text a search term was found in. */
export type SearchHitKind = 'DESCRIPTION' | 'STEP' | 'WRAPUP' | 'NOTE'

/** One place a search term was found, with the words around it. */
export interface SearchHit {
  readonly kind: SearchHitKind
  /** The task to open: the one the text is on, or for a note the first task it is on. */
  readonly taskId: TaskId | null
  readonly stepId: TaskStepId | null
  readonly documentId: DocumentId | null
  /** What the record is called: the task's, the step's or the note's title. */
  readonly title: string
  /** The anchor of the task the text is on. */
  readonly where: string
  readonly excerpt: string
}

export const SEARCH_HIT_LABEL: Readonly<Record<SearchHitKind, string>> = {
  DESCRIPTION: 'Description',
  STEP: 'Step',
  WRAPUP: 'Wrapup',
  NOTE: 'Note'
}

/** Below this many characters, or for a query that is only anchors, the text is not searched. */
export const SEARCH_TERM_MIN = 3

export function isTextQuery(query: string): boolean {
  const term = query.trim()
  return term.length >= SEARCH_TERM_MIN && !/\b(company|project|task):/i.test(term)
}
