import { onUnmounted, ref, watch, type Ref } from 'vue'

/** What the dock keeps between itself and the window edges. Mirrors `right-4` / `bottom-2.5`. */
const GUTTER_RIGHT_PX = 16
const GUTTER_BOTTOM_PX = 10

/** And between itself and the nearest control, so the two read as neighbours on one line. */
const CLEARANCE_PX = 12

const WIDTH_PROPERTY = '--dock-lane-width'
const HEIGHT_PROPERTY = '--dock-lane-height'

/**
 * The bottom right corner, measured while the running dock is standing in it.
 *
 * The dock is fixed to that corner and the step composer ends in it, so the pill sat on top of
 * the button that adds a step: with a timer running the only way to add one was to stop the
 * timer. A floating element is allowed to cover the canvas and never a control.
 *
 * Neither side guesses the other's size. The dock publishes the lane it occupies as two custom
 * properties on the root element and whatever ends in that corner reserves it: the composer
 * opens its right edge by the width, the toaster lifts by the height. Measured rather than
 * assumed, because the pill grows with the number of running tasks and again when a clock rolls
 * past an hour. The lane is absent while nothing runs, which is most of the time, so nothing
 * moves on a quiet day.
 *
 * The element handed back is the pill alone, never the wrapper: the expanded list is 300px of
 * panel that would otherwise shove the composer aside every time it was opened.
 */
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
    // ResizeObserver reports the element once on observe, so the first lane needs no second path.
    observer = new ResizeObserver(() => publish(element))
    observer.observe(element)
  })

  onUnmounted(() => {
    observer?.disconnect()
    clear()
  })

  return pill
}
