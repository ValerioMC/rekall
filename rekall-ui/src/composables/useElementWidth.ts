import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

/** The width of an element in pixels, kept current as it is resized; zero until it is mounted. */
export function useElementWidth(target: Ref<HTMLElement | null>): Ref<number> {
  const width = ref(0)
  let watcher: ResizeObserver | null = null

  onMounted(() => {
    const element = target.value
    if (!element) return
    width.value = element.clientWidth
    if (typeof ResizeObserver === 'undefined') return
    watcher = new ResizeObserver(() => (width.value = element.clientWidth))
    watcher.observe(element)
  })
  onBeforeUnmount(() => watcher?.disconnect())

  return width
}
