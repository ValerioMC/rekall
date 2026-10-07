import { ref, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { revealTableSource, transformTable } from '@/common/live-preview/table-edit'
import { tableOperationFor, type TableMenuAction } from '@/common/live-preview/table-menu'
import type { TableMenuRequest } from '@/common/live-preview/table-widget'

/** The menu a right-click on a drawn table opens, and the edit its choice makes in the editor. */
export function useTableContextMenu(editor: Ref<ExposeParam | null>): {
  request: Ref<TableMenuRequest | null>
  open: (event: Event) => void
  choose: (action: TableMenuAction) => void
  close: () => void
} {
  const request = ref<TableMenuRequest | null>(null)

  function open(event: Event): void {
    if (event instanceof CustomEvent) request.value = event.detail as TableMenuRequest
  }

  function choose(action: TableMenuAction): void {
    const target = request.value
    const view = editor.value?.getEditorView()
    request.value = null
    if (!target || !view) return
    if (action === 'editSource') revealTableSource(view, target.tablePosition)
    else transformTable(view, target.tablePosition, tableOperationFor(action, target.cell))
  }

  return { request, open, choose, close: () => (request.value = null) }
}
