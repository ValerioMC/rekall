import { ref, watch, type Ref } from 'vue'
import { fetchTaskChanges } from '@/features/commits/api/taskChanges.api'
import type { TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskId } from '@/model/branded'

export type TaskChangesStatus = 'loading' | 'ready' | 'failed'

/**
 * The changes of the task in view, read when the task is shown and again whenever `refreshKey`
 * changes (a commit was logged) or `reload` is called. An answer for a task no longer in view is dropped.
 */
export function useTaskChanges(
  taskId: () => TaskId,
  refreshKey: () => unknown
): { changes: Ref<TaskChanges | null>; status: Ref<TaskChangesStatus>; failure: Ref<string | null>; reload: () => Promise<void> } {
  const changes = ref<TaskChanges | null>(null)
  const status = ref<TaskChangesStatus>('loading')
  const failure = ref<string | null>(null)

  async function reload(): Promise<void> {
    const wanted = taskId()
    if (changes.value === null) status.value = 'loading'
    try {
      const read = await fetchTaskChanges(wanted)
      if (wanted !== taskId()) return
      changes.value = read
      failure.value = null
      status.value = 'ready'
    } catch (error) {
      if (wanted !== taskId()) return
      failure.value = error instanceof Error ? error.message : 'The changes could not be read.'
      status.value = 'failed'
    }
  }

  watch(taskId, () => {
    changes.value = null
    status.value = 'loading'
  })
  watch([taskId, refreshKey], () => void reload(), { immediate: true })

  return { changes, status, failure, reload }
}
