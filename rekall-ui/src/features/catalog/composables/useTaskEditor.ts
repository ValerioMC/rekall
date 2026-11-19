import { ref } from 'vue'
import { taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId } from '@/model/branded'

export interface TaskCreation {
  readonly projectId?: ProjectId
  readonly title?: string
}

const editing = ref<RecordDraft | null>(null)

/** The record dialog's draft for a new task, or for the task in view. One draft for the whole console, so any page can open the dialog it hosts. */
export function useTaskEditor() {
  function beginCreate(creation: TaskCreation = {}): void {
    const store = useConsoleStore()
    const projectId =
      creation.projectId ?? store.scopeProject ?? store.selectedTask?.projectId ?? store.scopedProjects[0]?.id
    if (!projectId) return
    editing.value = taskDraft(projectId, undefined, null, creation.title)
  }

  function editSelected(): void {
    const task = useConsoleStore().selectedTask
    if (task) editing.value = taskDraft(task.projectId, task)
  }

  function close(): void {
    editing.value = null
  }

  return { editing, beginCreate, editSelected, close }
}
