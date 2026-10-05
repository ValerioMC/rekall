import { computed, onBeforeUnmount, onMounted, ref, type ComputedRef } from 'vue'
import { viewportEditorHeight } from '@/common/config/editor-height'

/** The editor's least height for a pane that fills the window, following the window as it is resized. */
export function useViewportEditorHeight(chromeHeight: number, floor: number): ComputedRef<string> {
  const viewportHeight = ref(window.innerHeight)
  const track = (): void => {
    viewportHeight.value = window.innerHeight
  }

  onMounted(() => window.addEventListener('resize', track))
  onBeforeUnmount(() => window.removeEventListener('resize', track))

  return computed(() => viewportEditorHeight(viewportHeight.value, chromeHeight, floor))
}
