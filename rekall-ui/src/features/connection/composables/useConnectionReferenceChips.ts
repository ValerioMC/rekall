import { nextTick, onMounted, watch, type Ref } from 'vue'
import { paintConnectionReferences } from '@/common/markdown/connection-reference'
import type { ProjectConnection } from '@/features/connection/model/connection'

/**
 * Keeps the `[[connection:label]]` chips of a read view true to the project's bindings: painted when
 * the page renders (`paint`) and again whenever the bindings change.
 */
export function useConnectionReferenceChips(
  root: Ref<HTMLElement | null>,
  connections: () => readonly ProjectConnection[] | null
): { paint: () => void } {
  function paint(): void {
    void nextTick(() => {
      if (root.value) paintConnectionReferences(root.value, connections())
    })
  }

  onMounted(paint)
  watch(connections, paint)
  return { paint }
}
