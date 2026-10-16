import { onBeforeUnmount, type Ref } from 'vue'

/** A press on the canvas itself, not on a step, a route or a button, is the start of a pan. */
export function isCanvasBackground(
  target: EventTarget | null,
  scroller: HTMLElement,
  surface: HTMLElement | null
): boolean {
  return target === scroller || target === surface
}

/** Dragging the empty canvas scrolls the graph under the pointer, in canvas pixels. */
export function useFlowPan(scroller: Ref<HTMLElement | null>, surface: Ref<HTMLElement | null>) {
  let stop: (() => void) | null = null

  function startPan(press: PointerEvent): void {
    const element = scroller.value
    if (!element || press.button !== 0 || !isCanvasBackground(press.target, element, surface.value)) return
    const originLeft = element.scrollLeft
    const originTop = element.scrollTop
    const onMove = (event: PointerEvent): void => {
      element.scrollLeft = originLeft - (event.clientX - press.clientX)
      element.scrollTop = originTop - (event.clientY - press.clientY)
    }
    const onEnd = (): void => stop?.()
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onEnd)
    window.addEventListener('pointercancel', onEnd)
    stop = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onEnd)
      window.removeEventListener('pointercancel', onEnd)
      stop = null
    }
  }

  onBeforeUnmount(() => stop?.())

  return { startPan }
}
