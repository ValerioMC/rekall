import { nextTick, onMounted, onUnmounted, ref, watch, type Ref, type WatchSource } from 'vue'

export interface EnergyStream {
  top: number
  height: number
}

/** Where a row's node sits below the row's top edge. */
const NODE_CENTER_OFFSET = 18

/**
 * The lit stretch of a checklist's rail: from the first claimed or done step down to the one a
 * session is running, so the eye follows where the work has got to. None when no step runs, or
 * when nothing finished sits above the running one. Measured from the rendered rows, again on
 * every resize and on every change in `remeasureOn`.
 */
export function useEnergyStream(list: Ref<HTMLElement | null>, remeasureOn: WatchSource[]): Ref<EnergyStream | null> {
  const stream = ref<EnergyStream | null>(null)

  function measure(): void {
    const rows = list.value ? Array.from(list.value.querySelectorAll<HTMLElement>('li[data-testid="step-row"]')) : []
    const runningIndex = rows.findIndex((row) => row.dataset.stepState === 'RUNNING')
    const startIndex = rows.findIndex((row) => row.dataset.stepState === 'CLAIMED' || row.dataset.stepState === 'DONE')
    const startRow = rows[startIndex]
    const runningRow = rows[runningIndex]
    if (!startRow || !runningRow || runningIndex < 1 || startIndex >= runningIndex) {
      stream.value = null
      return
    }
    const top = startRow.offsetTop + NODE_CENTER_OFFSET
    const height = runningRow.offsetTop + NODE_CENTER_OFFSET - top
    stream.value = height > 4 ? { top, height } : null
  }

  function scheduleMeasure(): void {
    void nextTick(measure)
  }

  const rowObserver = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(() => measure()) : null

  watch(list, (element) => {
    rowObserver?.disconnect()
    if (element) rowObserver?.observe(element)
    scheduleMeasure()
  })
  watch(remeasureOn, scheduleMeasure)

  onMounted(() => {
    scheduleMeasure()
    window.addEventListener('resize', scheduleMeasure)
  })
  onUnmounted(() => {
    rowObserver?.disconnect()
    window.removeEventListener('resize', scheduleMeasure)
  })

  return stream
}
