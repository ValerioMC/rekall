import { computed, watch, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useToastStore } from '@/stores/toast.store'
import type { ProjectConnection } from '@/features/connection/model/connection'

/**
 * The connection bindings of the selected task's project, loaded when the task is selected. `null`
 * until they are known, so a reference is never called broken just because the list has not arrived.
 */
export function useSelectedProjectConnections(): ComputedRef<readonly ProjectConnection[] | null> {
  const { selectedTask } = storeToRefs(useConsoleStore())
  const store = useConnectionsStore()
  const toast = useToastStore()
  const projectId = computed(() => selectedTask.value?.projectId ?? null)

  watch(
    projectId,
    (id) => {
      if (id && !store.isProjectLoaded(id)) void store.loadProject(id).catch((caught: unknown) => toast.notifyError(caught))
    },
    { immediate: true }
  )

  return computed(() =>
    projectId.value && store.isProjectLoaded(projectId.value) ? store.projectConnectionsOf(projectId.value) : null
  )
}
