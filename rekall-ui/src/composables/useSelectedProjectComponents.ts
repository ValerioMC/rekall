import { computed, watch, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import { useToastStore } from '@/stores/toast.store'
import type { ProjectComponent } from '@/model/project-component'

/**
 * The components of the selected task's project, loaded when the task is selected. `null` until
 * they are known, so a reference is never called broken just because the list has not arrived.
 */
export function useSelectedProjectComponents(): ComputedRef<readonly ProjectComponent[] | null> {
  const { selectedTask } = storeToRefs(useConsoleStore())
  const store = useProjectComponentsStore()
  const toast = useToastStore()
  const projectId = computed(() => selectedTask.value?.projectId ?? null)

  watch(
    projectId,
    (id) => {
      if (id) void store.ensureLoaded(id).catch((caught: unknown) => toast.notifyError(caught))
    },
    { immediate: true }
  )

  return computed(() => (projectId.value && store.isLoaded(projectId.value) ? store.componentsOf(projectId.value) : null))
}
