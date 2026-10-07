import { syntaxTree } from '@codemirror/language'
import type { EditorState } from '@codemirror/state'
import type { EditorView } from '@codemirror/view'
import { caretInto } from '@/common/live-preview/editor-focus'
import {
  applyOperation,
  cellChange,
  parseTable,
  serializeTable,
  toGrid,
  type CellAddress,
  type TableOperation
} from '@/common/live-preview/markdown-table'

/**
 * Edits to a drawn table, written into the editor's text. A table is found by the position its
 * widget starts at, since the widget itself survives edits that move it.
 */

interface TableRange {
  from: number
  to: number
}

export function editTableCell(
  view: EditorView,
  tablePosition: number,
  address: CellAddress,
  text: string
): void {
  const range = tableRangeAt(view.state, tablePosition)
  if (!range) return
  const change = cellChange(view.state.sliceDoc(range.from, range.to), address, text)
  const from = range.from + change.from
  const to = range.from + change.to
  if (view.state.sliceDoc(from, to) === change.insert) return
  view.dispatch({ changes: { from, to, insert: change.insert }, userEvent: 'input.table' })
}

export function transformTable(view: EditorView, tablePosition: number, operation: TableOperation): void {
  const range = tableRangeAt(view.state, tablePosition)
  if (!range) return
  const grid = applyOperation(toGrid(parseTable(view.state.sliceDoc(range.from, range.to))), operation)
  view.dispatch({ changes: { ...range, insert: serializeTable(grid) }, userEvent: 'input.table' })
}

/** Puts the caret in the table's markdown, which takes its widget away until the caret leaves. */
export function revealTableSource(view: EditorView, tablePosition: number): void {
  const range = tableRangeAt(view.state, tablePosition)
  if (!range) return
  view.focus()
  view.dispatch(caretInto(range.from))
}

/** The widget starts at its line's beginning; an indented table's node starts a little later. */
function tableRangeAt(state: EditorState, position: number): TableRange | undefined {
  let found: TableRange | undefined
  syntaxTree(state).iterate({
    from: position,
    to: state.doc.lineAt(position).to,
    enter: (node) => {
      if (found) return false
      if (node.name !== 'Table') return
      found = { from: node.from, to: node.to }
      return false
    }
  })
  return found
}
