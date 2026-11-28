import { computed, ref, type Ref } from 'vue'
import { toggledSet } from '@/common/config/persisted-set'
import { nextSort } from '@/features/project/lib/project-task-rules'
import {
  columnAbsence,
  columnsFor,
  SIDE_PREVIEW_MIN_WIDTH,
  SIDE_PREVIEW_SPAN
} from '@/features/project/lib/task-columns'
import type { ProjectTaskPreferences } from '@/common/config/project-task-preferences'
import type { ListedTask, OptionalTaskColumn, ProjectTaskSortKey } from '@/features/project/model/project-task-list'

/**
 * How the task list is drawn: density, order, page size, the columns taken away and the side
 * preview. The columns on screen follow from these and from the width the list has.
 */
export function useTaskLayout(remembered: ProjectTaskPreferences, rows: Ref<readonly ListedTask[]>) {
  const density = ref(remembered.density)
  const sort = ref(remembered.sort)
  const pageSize = ref(remembered.pageSize)
  const hiddenColumns = ref<ReadonlySet<OptionalTaskColumn>>(new Set(remembered.hiddenColumns))
  const sidePreview = ref(remembered.sidePreview)
  const width = ref(0)

  const sidePreviewFits = computed(() => width.value >= SIDE_PREVIEW_MIN_WIDTH)
  const previewsBeside = computed(() => sidePreview.value && sidePreviewFits.value)
  const columnChoice = computed(() => ({
    width: previewsBeside.value ? width.value - SIDE_PREVIEW_SPAN : width.value,
    hidden: hiddenColumns.value,
    rows: rows.value
  }))
  const columns = computed(() => columnsFor(columnChoice.value))

  function sortBy(key: ProjectTaskSortKey): void {
    sort.value = nextSort(sort.value, key)
  }

  function toggleColumn(column: OptionalTaskColumn): void {
    hiddenColumns.value = toggledSet(hiddenColumns.value, column)
  }

  function absenceOf(column: OptionalTaskColumn): string | null {
    return columnAbsence(column, columnChoice.value)
  }

  /** The ledger reports the width it has, which decides the columns and whether the preview fits beside it. */
  function measure(listWidth: number): void {
    width.value = listWidth
  }

  return { density, sort, pageSize, hiddenColumns, sidePreview, sidePreviewFits, previewsBeside, columns, sortBy, toggleColumn, absenceOf, measure }
}
