import { computed, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'

/** The other tasks of the selected task's project, the ones its text can name as `[[task:label]]`. `null` with no task selected. */
export function useSelectedProjectTasks(): ComputedRef<readonly Task[] | null> {
  const { selectedTask, tasks } = storeToRefs(useConsoleStore())
  return computed(() => {
    const current = selectedTask.value
    if (!current) return null
    return tasks.value.filter((task) => task.projectId === current.projectId && task.id !== current.id)
  })
}
