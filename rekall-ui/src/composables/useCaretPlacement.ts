import { computed, onMounted, onUnmounted, ref, type ComputedRef } from 'vue'

export interface CaretBox {
  readonly left: number
  readonly top: number
  readonly bottom: number
}

/** The size a popover asks for; it takes less when the window has no room. */
export interface PanelSize {
  readonly width: number
  readonly height: number
}

interface Placement {
  left: number
  width: number
  height: number
  top?: number
  bottom?: number
}

const MIN_BELOW = 260
const GUTTER = 8
const GAP = 6

/**
 * A popover hung from the writing caret: under it when there is room, over it when there is not,
 * kept inside the window, and placed again on every resize and scroll. The height is the asked one rather
 * than fitted to the content, so the panel does not jump as a filter narrows a list. A caret that
 * can no longer be measured calls `onLost`.
 */
export function useCaretPlacement(
  locate: () => CaretBox | null,
  onLost: () => void,
  size: PanelSize
): ComputedRef<Record<string, string | undefined>> {
  const placement = ref<Placement>({ left: 0, ...size })

  function place(): void {
    const caret = locate()
    if (!caret) {
      onLost()
      return
    }
    const width = Math.min(size.width, window.innerWidth - 2 * GUTTER)
    const left = Math.max(GUTTER, Math.min(caret.left - 22, window.innerWidth - width - GUTTER))
    const below = window.innerHeight - caret.bottom - GAP - GUTTER
    const above = caret.top - GAP - GUTTER
    placement.value =
      below >= MIN_BELOW || below >= above
        ? { left, width, height: Math.min(size.height, below), top: caret.bottom + GAP }
        : { left, width, height: Math.min(size.height, above), bottom: window.innerHeight - caret.top + GAP }
  }

  onMounted(() => {
    place()
    window.addEventListener('resize', place)
    window.addEventListener('scroll', place, true)
  })
  onUnmounted(() => {
    window.removeEventListener('resize', place)
    window.removeEventListener('scroll', place, true)
  })

  return computed(() => ({
    left: `${placement.value.left}px`,
    width: `${placement.value.width}px`,
    height: `${placement.value.height}px`,
    top: placement.value.top === undefined ? undefined : `${placement.value.top}px`,
    bottom: placement.value.bottom === undefined ? undefined : `${placement.value.bottom}px`,
    transformOrigin: placement.value.top === undefined ? 'bottom left' : 'top left'
  }))
}
