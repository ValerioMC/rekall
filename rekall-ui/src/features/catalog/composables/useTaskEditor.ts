import { ref } from 'vue'
import { taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import { useConsoleStore } from '@/stores/console.store'

/** The record dialog's draft for a new task, or for the task in view. */
export function useTaskEditor() {
  const store = useConsoleStore()
  const editing = ref<RecordDraft | null>(null)

  function beginCreate(): void {
    const projectId = store.scopeProject ?? store.selectedTask?.projectId ?? store.scopedProjects[0]?.id
    if (!projectId) return
    editing.value = taskDraft(projectId)
  }

  function editSelected(): void {
    const task = store.selectedTask
    if (task) editing.value = taskDraft(task.projectId, task)
  }

  function close(): void {
    editing.value = null
  }

  return { editing, beginCreate, editSelected, close }
}
