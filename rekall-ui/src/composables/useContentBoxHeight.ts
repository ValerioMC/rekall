import { computed, onBeforeUnmount, onMounted, ref, type ComputedRef, type Ref } from 'vue'
import { contentBoxHeight } from '@/common/config/editor-height'

/** The height `target` offers its content, as a CSS length, following the element as the window or layout resizes it. */
export function useContentBoxHeight(target: Ref<HTMLElement | null>, floor: number): ComputedRef<string> {
  const measured = ref<string | null>(null)

  function measure(): void {
    const element = target.value
    if (!element) return
    const style = window.getComputedStyle(element)
    measured.value = contentBoxHeight(element.clientHeight, parseFloat(style.paddingTop) || 0, parseFloat(style.paddingBottom) || 0, floor)
  }

  let sizeWatcher: ResizeObserver | null = null
  onMounted(() => {
    measure()
    if (typeof ResizeObserver === 'undefined' || !target.value) return
    sizeWatcher = new ResizeObserver(measure)
    sizeWatcher.observe(target.value)
  })
  onBeforeUnmount(() => sizeWatcher?.disconnect())

  return computed(() => measured.value ?? `${floor}px`)
}
