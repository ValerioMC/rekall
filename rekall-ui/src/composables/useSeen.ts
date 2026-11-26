import { onMounted, onUnmounted, ref, type Ref } from 'vue'

/** True once the element has been in view, and from then on, so what it shows is not asked for before. */
export function useSeen(element: Readonly<Ref<Element | null>>): Readonly<Ref<boolean>> {
  const seen = ref(false)
  let observer: IntersectionObserver | null = null

  function stop(): void {
    observer?.disconnect()
    observer = null
  }

  onMounted(() => {
    if (!element.value) return
    observer = new IntersectionObserver((entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return
      seen.value = true
      stop()
    })
    observer.observe(element.value)
  })
  onUnmounted(stop)

  return seen
}
