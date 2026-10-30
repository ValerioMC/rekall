import { onScopeDispose, ref, watch, type Ref } from 'vue'

const PAINT_EVERY_MS = 120

/**
 * The text of a message that is still arriving, repainted at most every 120 ms so a markdown
 * render is not redone for every few words. The complete text shows at once.
 */
export function useStreamedText(text: () => string, complete: () => boolean): Ref<string> {
  const shown = ref(text())
  let timer: ReturnType<typeof setTimeout> | null = null

  function cancelPaint(): void {
    if (timer) clearTimeout(timer)
    timer = null
  }

  function paint(): void {
    timer = null
    shown.value = text()
  }

  watch([text, complete], () => {
    if (complete()) {
      cancelPaint()
      shown.value = text()
      return
    }
    if (!timer) timer = setTimeout(paint, PAINT_EVERY_MS)
  })

  onScopeDispose(cancelPaint)

  return shown
}
