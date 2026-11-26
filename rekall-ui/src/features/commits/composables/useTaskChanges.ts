import { onScopeDispose, ref, watch } from 'vue'
import { fetchTaskChanges } from '@/features/commits/api/taskChanges.api'
import type { TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskId } from '@/model/branded'

export type TaskChangesStatus = 'loading' | 'ready' | 'failed'

const LOADING_DELAY = 200

/**
 * The changes of the task in view, read when the task is shown and again whenever `refreshKey`
 * changes (a commit was logged) or `reload` is called. An answer for a task no longer in view is dropped.
 */
export function useTaskChanges(
  taskId: () => TaskId,
  refreshKey: () => unknown
) {
  const changes = ref<TaskChanges | null>(null)
  const status = ref<TaskChangesStatus>('loading')
  const failure = ref<string | null>(null)
  const showLoading = ref(false)
  let readVersion = 0
  let loadingTimer: ReturnType<typeof setTimeout> | undefined

  function clearLoadingDelay(): void {
    clearTimeout(loadingTimer)
    loadingTimer = undefined
    showLoading.value = false
  }

  function delayLoadingMessage(): void {
    status.value = 'loading'
    loadingTimer = setTimeout(() => (showLoading.value = true), LOADING_DELAY)
  }

  async function reload(): Promise<void> {
    const wanted = taskId()
    const version = ++readVersion
    clearLoadingDelay()
    if (changes.value === null) delayLoadingMessage()
    try {
      const read = await fetchTaskChanges(wanted)
      if (version !== readVersion || wanted !== taskId()) return
      changes.value = read
      failure.value = null
      status.value = 'ready'
    } catch (error) {
      if (version !== readVersion || wanted !== taskId()) return
      failure.value = error instanceof Error ? error.message : 'The changes could not be read.'
      status.value = 'failed'
    } finally {
      if (version === readVersion) clearLoadingDelay()
    }
  }

  watch(taskId, () => {
    changes.value = null
    status.value = 'loading'
  })
  watch([taskId, refreshKey], () => void reload(), { immediate: true })
  onScopeDispose(() => {
    readVersion++
    clearLoadingDelay()
  })

  return { changes, status, failure, showLoading, reload }
}
