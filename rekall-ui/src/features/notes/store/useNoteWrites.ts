import {
  createDocument as apiCreateDocument,
  deleteDocument as apiDeleteDocument,
  updateDocument as apiUpdateDocument
} from '@/features/notes/api/documents.api'
import type { RekallDocument } from '@/features/notes/model/document'
import type { DocumentId, TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import type { useSelectedTask } from '../../../stores/console/useSelectedTask'
import type { useTimers } from '../../time/store/useTimers'
import { replaceById, trackSave } from '../../../stores/console/rows'

type NotePatch = Partial<Pick<RekallDocument, 'title' | 'kind' | 'bodyMarkdown' | 'contextMode'>>

/** Whether a save changes what the note says, which is working on its task. */
function writesContent(current: RekallDocument, patch: NotePatch): boolean {
  return (
    (patch.title !== undefined && patch.title !== current.title) ||
    (patch.kind !== undefined && patch.kind !== current.kind) ||
    (patch.bodyMarkdown !== undefined && patch.bodyMarkdown !== current.bodyMarkdown)
  )
}

/** Notes written in the console. Tasks count their notes, so a write that adds or deletes one re-reads them. */
export function useNoteWrites(
  state: ConsoleState,
  selected: ReturnType<typeof useSelectedTask>,
  refreshes: ReturnType<typeof useConsoleRefreshes>,
  timers: ReturnType<typeof useTimers>
) {
  const { documents, selectedDocId, paneFocus, saveState } = state

  /** A note is written on one task and opens in the editor. */
  async function createNote(taskId: TaskId, title?: string): Promise<void> {
    const created = await apiCreateDocument({
      title: title?.trim() || 'untitled.md',
      kind: 'notes',
      bodyMarkdown: '',
      taskId
    })
    documents.value = [created, ...documents.value]
    selectedDocId.value = created.id
    paneFocus.value = 'note'
    await refreshes.refreshTasks()
    await timers.resumeTimerOnWork(taskId)
  }

  async function saveNote(id: DocumentId, patch: NotePatch): Promise<void> {
    const current = documents.value.find((document) => document.id === id)
    if (!current) return
    await trackSave(saveState, async () => {
      const saved = await apiUpdateDocument(id, {
        title: patch.title ?? current.title,
        kind: patch.kind ?? current.kind,
        bodyMarkdown: patch.bodyMarkdown ?? current.bodyMarkdown,
        taskId: current.taskId,
        contextMode: patch.contextMode ?? current.contextMode
      })
      documents.value = replaceById(documents.value, saved)
    })
    if (writesContent(current, patch)) await timers.resumeTimerOnWork(current.taskId)
  }

  async function deleteNote(id: DocumentId): Promise<void> {
    await apiDeleteDocument(id)
    documents.value = documents.value.filter((document) => document.id !== id)
    if (selectedDocId.value === id) {
      selectedDocId.value = selected.taskDocuments.value[0]?.id ?? null
    }
    await refreshes.refreshTasks()
  }

  /**
   * A session wrote a note onto a task. The event only names it, so the notes are fetched again,
   * and the tasks with them for their note count. Nothing is selected: the reader stays where
   * they were.
   */
  async function applyNoteWritten(): Promise<void> {
    await Promise.all([refreshes.refreshDocuments(), refreshes.refreshTasks()])
  }

  return {
    createNote,
    saveNote,
    deleteNote,
    applyNoteWritten
  }
}
