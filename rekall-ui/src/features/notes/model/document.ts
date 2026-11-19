import type { DocumentId, TaskId } from '../../../model/branded'

export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export interface RekallDocument {
  readonly id: DocumentId
  readonly title: string
  readonly kind: string
  readonly bodyMarkdown: string
  /** The one task the note belongs to and goes with. */
  readonly taskId: TaskId
  /** In full with its task, or as a reference a session loads by `anchor` when it needs it. */
  readonly contextMode: DocumentContextMode
  /** `note:` and the first characters of its id: what a session loads a reference note by. */
  readonly anchor: string
  readonly updatedAt: string
}

export type DocumentContextMode = 'FULL' | 'REFERENCE'

/** A note a session wrote onto a task (`note` SSE event). */
export interface NoteStreamEvent {
  readonly taskId: TaskId
  readonly documentId: DocumentId
}
