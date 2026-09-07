import { onUnmounted, ref, watch, type Ref } from 'vue'

const GUTTER_RIGHT_PX = 16
const GUTTER_BOTTOM_PX = 10

const CLEARANCE_PX = 12

const WIDTH_PROPERTY = '--dock-lane-width'
const HEIGHT_PROPERTY = '--dock-lane-height'

export function useDockLane(): Ref<HTMLElement | null> {
  const pill = ref<HTMLElement | null>(null)
  const root = document.documentElement
  let observer: ResizeObserver | null = null

  function clear(): void {
    root.style.removeProperty(WIDTH_PROPERTY)
    root.style.removeProperty(HEIGHT_PROPERTY)
  }

  function publish(element: HTMLElement): void {
    const box = element.getBoundingClientRect()
    root.style.setProperty(WIDTH_PROPERTY, `${GUTTER_RIGHT_PX + box.width + CLEARANCE_PX}px`)
    root.style.setProperty(HEIGHT_PROPERTY, `${GUTTER_BOTTOM_PX + box.height + CLEARANCE_PX}px`)
  }

  watch(pill, (element) => {
    observer?.disconnect()
    observer = null
    if (!element) {
      clear()
      return
    }
    observer = new ResizeObserver(() => publish(element))
    observer.observe(element)
  })

  onUnmounted(() => {
    observer?.disconnect()
    clear()
  })

  return pill
}
