import { nextTick, onMounted, watch, type Ref } from 'vue'
import { paintComponentReferences } from '@/common/markdown/component-reference'
import type { ProjectComponent } from '@/features/project/model/project-component'

/**
 * Keeps the `[[Name]]` chips of a read view true to the project's components: painted when the
 * page renders (`paint`) and again whenever the components change, so a rename shows without a reload.
 */
export function useComponentReferenceChips(
  root: Ref<HTMLElement | null>,
  components: () => readonly ProjectComponent[] | null
): { paint: () => void } {
  function paint(): void {
    void nextTick(() => {
      if (root.value) paintComponentReferences(root.value, components())
    })
  }

  onMounted(paint)
  watch(components, paint)
  return { paint }
}
