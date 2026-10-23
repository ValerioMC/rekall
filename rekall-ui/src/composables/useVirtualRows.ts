import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'

export interface RowRange {
  readonly start: number
  readonly end: number
}

/** The rows of a fixed height that cross the viewport, widened by `overscan` on each side. */
export function visibleRange(scrollTop: number, viewportHeight: number, rowHeight: number, count: number, overscan: number): RowRange {
  const first = Math.floor(scrollTop / rowHeight)
  const shown = Math.ceil(viewportHeight / rowHeight)
  return { start: Math.max(0, first - overscan), end: Math.min(count, first + shown + overscan) }
}

/**
 * A long list drawn a window at a time: only the rows in view are in the DOM, inside a spacer as tall
 * as the whole list, so a thousand dependencies scroll as lightly as forty. Rows share one height.
 */
export function useVirtualRows<Row>(rows: Ref<readonly Row[]>, rowHeight: number, overscan = 10) {
  const viewport = ref<HTMLElement | null>(null)
  const scrollTop = ref(0)
  const viewportHeight = ref(0)
  let observer: ResizeObserver | null = null

  function measure(): void {
    viewportHeight.value = viewport.value?.clientHeight ?? 0
    scrollTop.value = viewport.value?.scrollTop ?? 0
  }

  watch(viewport, (element) => {
    observer?.disconnect()
    if (!element) return
    measure()
    if (typeof ResizeObserver === 'undefined') return
    observer = new ResizeObserver(measure)
    observer.observe(element)
  })
  onScopeDispose(() => observer?.disconnect())

  const range = computed(() => visibleRange(scrollTop.value, viewportHeight.value || rowHeight * 30, rowHeight, rows.value.length, overscan))
  const visible = computed(() => rows.value.slice(range.value.start, range.value.end).map((row, offset) => ({ row, index: range.value.start + offset })))
  const totalHeight = computed(() => rows.value.length * rowHeight)
  const offsetTop = computed(() => range.value.start * rowHeight)

  function scrollToTop(): void {
    if (viewport.value) viewport.value.scrollTop = 0
    scrollTop.value = 0
  }

  return { viewport, visible, totalHeight, offsetTop, onScroll: measure, scrollToTop }
}
