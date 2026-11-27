import { nextTick, onMounted, type Ref } from 'vue'
import { keepShortColumnsWhole, TABLE_FRAME_CLASS } from '@/common/markdown/table-layout'

/** Keeps the short columns of a read view's tables on one line, each time the page renders. */
export function useShortTableColumns(root: Ref<HTMLElement | null>): { paint: () => void } {
  function paint(): void {
    void nextTick(() => {
      root.value?.querySelectorAll<HTMLTableElement>(`.${TABLE_FRAME_CLASS} > table`).forEach(keepShortColumnsWhole)
    })
  }

  onMounted(paint)
  return { paint }
}
