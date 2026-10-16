import {
  createDocument as apiCreateDocument,
  deleteDocument as apiDeleteDocument,
  updateDocument as apiUpdateDocument
} from '@/features/notes/api/documents.api'
import type { RekallDocument } from '@/features/catalog/model/catalog'
import type { DocumentId, TaskId } from '@/model/branded'
import { GLOBAL_SCOPE, type NoteScope } from '@/features/notes/model/note-scope'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import type { useSelectedTask } from '../../../stores/console/useSelectedTask'
import type { useTimers } from '../../time/store/useTimers'
import { replaceById, trackSave } from '../../../stores/console/rows'

/** What a new note starts from: where it lives, which of that scope's tasks it is on, its name. */
export interface NoteDraft {
  readonly scope: NoteScope
  readonly taskIds?: readonly TaskId[]
  readonly title?: string
}

type NotePatch = Partial<
  Pick<RekallDocument, 'title' | 'kind' | 'bodyMarkdown' | 'contextMode' | 'scope'>
> & {
  taskIds?: readonly TaskId[]
}

/** Whether a save changes what the note says, which is working on its task, rather than where it sits. */
function writesContent(current: RekallDocument, patch: NotePatch): boolean {
  return (
    (patch.title !== undefined && patch.title !== current.title) ||
    (patch.kind !== undefined && patch.kind !== current.kind) ||
    (patch.bodyMarkdown !== undefined && patch.bodyMarkdown !== current.bodyMarkdown)
  )
}

/** Notes written in the console. Tasks count their notes, so a write that moves one re-reads them. */
export function useNoteWrites(
  state: ConsoleState,
  selected: ReturnType<typeof useSelectedTask>,
  refreshes: ReturnType<typeof useConsoleRefreshes>,
  timers: ReturnType<typeof useTimers>
) {
  const { tasks, documents, navMode, selectedTaskId, selectedDocId, paneFocus, noteComposerOpen, saveState } =
    state

  /** The scope a note made from a task starts in: that task's project. */
  function projectScopeOf(taskId: TaskId): NoteScope {
    const task = tasks.value.find((candidate) => candidate.id === taskId)
    return task ? { kind: 'PROJECT', id: task.projectId } : GLOBAL_SCOPE
  }

  /**
   * A note is born in a scope and on whichever of its tasks were picked, possibly none. The new
   * one opens in the editor; browsing notes, that also closes the composer that made it.
   */
  async function createNote(draft: NoteDraft): Promise<void> {
    const taskIds = draft.taskIds ?? []
    const created = await apiCreateDocument({
      title: draft.title?.trim() || 'untitled.md',
      kind: 'notes',
      bodyMarkdown: '',
      taskIds: [...taskIds],
      scope: draft.scope
    })
    documents.value = [created, ...documents.value]
    selectedDocId.value = created.id
    noteComposerOpen.value = false
    paneFocus.value = 'note'
    await refreshes.refreshTasks()
    if (taskIds.length) await timers.resumeTimerOnNote(taskIds)
  }

  async function saveNote(id: DocumentId, patch: NotePatch): Promise<void> {
    const current = documents.value.find((document) => document.id === id)
    if (!current) return
    await trackSave(saveState, async () => {
      const saved = await apiUpdateDocument(id, {
        title: patch.title ?? current.title,
        kind: patch.kind ?? current.kind,
        bodyMarkdown: patch.bodyMarkdown ?? current.bodyMarkdown,
        taskIds: patch.taskIds ?? current.tasks.map((ref) => ref.id),
        contextMode: patch.contextMode ?? current.contextMode,
        scope: patch.scope ?? current.scope
      })
      documents.value = replaceById(documents.value, saved)
    })
    if (patch.taskIds || patch.scope) await refreshes.refreshTasks()
    if (writesContent(current, patch))
      await timers.resumeTimerOnNote(patch.taskIds ?? current.tasks.map((ref) => ref.id))
  }

  async function attachNoteToTask(id: DocumentId, taskId: TaskId): Promise<void> {
    const current = documents.value.find((document) => document.id === id)
    if (!current || current.tasks.some((ref) => ref.id === taskId)) return
    await saveNote(id, { taskIds: [...current.tasks.map((ref) => ref.id), taskId] })
  }

  /**
   * Its scope keeps a note taken off every task. Browsing tasks, the editor shows a note on the
   * task in view; when that note leaves that task, the next note on it takes its place so the
   * columns keep agreeing.
   */
  async function detachNoteFromTask(id: DocumentId, taskId: TaskId): Promise<void> {
    const current = documents.value.find((document) => document.id === id)
    if (!current) return
    const remaining = current.tasks.filter((ref) => ref.id !== taskId).map((ref) => ref.id)
    if (remaining.length === current.tasks.length) return
    await saveNote(id, { taskIds: remaining })
    if (navMode.value === 'tasks' && selectedDocId.value === id && selectedTaskId.value === taskId) {
      selectedDocId.value = selected.taskDocuments.value[0]?.id ?? null
    }
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
    projectScopeOf,
    createNote,
    saveNote,
    attachNoteToTask,
    detachNoteFromTask,
    deleteNote,
    applyNoteWritten
  }
}
