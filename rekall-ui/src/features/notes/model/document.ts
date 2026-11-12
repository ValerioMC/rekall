import type { DocumentId, TaskId } from '../../../model/branded'
import type { TaskRef } from '../../catalog/model/catalog'
import type { NoteScope } from './note-scope'

export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export interface RekallDocument {
  readonly id: DocumentId
  readonly title: string
  readonly kind: string
  readonly bodyMarkdown: string
  readonly tasks: readonly TaskRef[]
  /** In full under every task it is on, or as a reference a session loads by `anchor` when it needs it. */
  readonly contextMode: DocumentContextMode
  /** Who owns the note: where it is listed and which tasks it may sit on. */
  readonly scope: NoteScope
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
