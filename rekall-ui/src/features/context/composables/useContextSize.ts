import { computed, onUnmounted, ref, watch, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { fetchContextSize } from '@/features/context/api/context.api'
import type { ContextSize } from '@/features/context/model/context'
import type { TaskId } from '@/model/branded'

const MEASURE_DEBOUNCE_MS = 400

/**
 * What `/rk` hands a session for a task, measured by the server. It is asked again, after a short
 * pause, whenever something that goes into the context changes; a stale answer is dropped.
 */
export function useContextSize(taskId: () => TaskId): { size: Ref<ContextSize | null>; failed: Ref<boolean> } {
  const store = useConsoleStore()
  const { selectedTask, selectedWrapup, taskDocuments, selectedTaskSteps } = storeToRefs(store)

  const size = ref<ContextSize | null>(null)
  const failed = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null
  let sequence = 0

  const signature = computed(() =>
    [
      taskId(),
      selectedTask.value?.updatedAt,
      selectedWrapup.value?.updatedAt,
      ...taskDocuments.value.map((document) => `${document.id}:${document.updatedAt}:${document.contextMode}`),
      ...selectedTaskSteps.value.map((step) => `${step.id}:${step.updatedAt}:${step.state}`)
    ].join('|')
  )

  async function measure(id: TaskId, at: number): Promise<void> {
    try {
      const measured = await fetchContextSize(id)
      if (at !== sequence) return
      size.value = measured
      failed.value = false
    } catch {
      if (at !== sequence) return
      failed.value = true
    }
  }

  watch(
    signature,
    () => {
      if (timer) clearTimeout(timer)
      const at = ++sequence
      const id = taskId()
      timer = setTimeout(() => void measure(id, at), MEASURE_DEBOUNCE_MS)
    },
    { immediate: true }
  )

  watch(taskId, () => {
    size.value = null
  })

  onUnmounted(() => {
    if (timer) clearTimeout(timer)
  })

  return { size, failed }
}
