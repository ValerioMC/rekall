import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

export interface ElementSize {
  readonly width: number
  readonly height: number
}

/** The size of an element in pixels, kept current as it is resized; zero until it is mounted. */
export function useElementSize(target: Ref<HTMLElement | null>): Ref<ElementSize> {
  const size = ref<ElementSize>({ width: 0, height: 0 })
  let watcher: ResizeObserver | null = null

  function measure(element: HTMLElement): void {
    size.value = { width: element.clientWidth, height: element.clientHeight }
  }

  onMounted(() => {
    const element = target.value
    if (!element) return
    measure(element)
    if (typeof ResizeObserver === 'undefined') return
    watcher = new ResizeObserver(() => measure(element))
    watcher.observe(element)
  })
  onBeforeUnmount(() => watcher?.disconnect())

  return size
}
