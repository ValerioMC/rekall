import { computed, watch, type ComputedRef } from 'vue'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import type { TaskDeviation } from '@/features/deviations/model/deviation'
import type { TaskId } from '@/model/branded'

/** The deviations of the task in view, read the first time the task is shown, and whether they have been. */
export function useTaskDeviations(taskId: ComputedRef<TaskId | null>): {
  deviations: ComputedRef<readonly TaskDeviation[]>
  isLoaded: ComputedRef<boolean>
} {
  const deviations = useDeviationsStore()

  watch(
    taskId,
    (id) => {
      if (id !== null && !deviations.isKnown(id)) void deviations.load(id).catch(() => undefined)
    },
    { immediate: true }
  )

  return {
    deviations: computed(() => (taskId.value === null ? [] : deviations.of(taskId.value))),
    isLoaded: computed(() => taskId.value !== null && deviations.isKnown(taskId.value))
  }
}
