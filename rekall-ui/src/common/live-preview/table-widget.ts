import { WidgetType, type EditorView } from '@codemirror/view'
import { renderInlineMarkdown } from '@/common/live-preview/inline-markdown'
import {
  parseTable,
  toGrid,
  type CellAddress,
  type ColumnAlignment
} from '@/common/live-preview/markdown-table'
import { editTableCell } from '@/common/live-preview/table-edit'

/** Raised on the editor when a cell is right-clicked; the editor's owner shows the menu. */
export const TABLE_MENU_EVENT = 'rk-table-menu'

export interface TableMenuRequest {
  x: number
  y: number
  tablePosition: number
  cell: CellAddress
  rowCount: number
  columnCount: number
}

/**
 * A GFM table drawn as a table whose cells are edited in place. A cell shows its markdown while it
 * is edited and its rendering otherwise; leaving it writes the text back into that cell only.
 * Listeners read everything from the DOM, because `updateDOM` keeps the element across edits.
 */
export class TableWidget extends WidgetType {
  constructor(readonly source: string) {
    super()
  }

  override eq(other: TableWidget): boolean {
    return other.source === this.source
  }

  override toDOM(view: EditorView): HTMLElement {
    const frame = document.createElement('div')
    frame.className = 'cm-rk-table'
    this.fill(frame)
    listen(frame, view)
    return frame
  }

  override updateDOM(dom: HTMLElement): boolean {
    this.fill(dom)
    return true
  }

  override ignoreEvent(): boolean {
    return true
  }

  private fill(frame: HTMLElement): void {
    const grid = toGrid(parseTable(this.source))
    const table = document.createElement('table')
    const head = table.createTHead().insertRow()
    grid.header.forEach((text, column) =>
      head.appendChild(cellElement('th', text, { row: 0, column }, grid.alignments[column]))
    )
    const body = table.createTBody()
    grid.rows.forEach((cells, index) => {
      const row = body.insertRow()
      cells.forEach((text, column) =>
        row.appendChild(cellElement('td', text, { row: index + 1, column }, grid.alignments[column]))
      )
    })
    frame.dataset.rows = String(grid.rows.length + 1)
    frame.dataset.columns = String(grid.header.length)
    frame.replaceChildren(table)
  }
}

function cellElement(
  tag: 'th' | 'td',
  text: string,
  address: CellAddress,
  alignment: ColumnAlignment | undefined
): HTMLElement {
  const cell = document.createElement(tag)
  cell.dataset.row = String(address.row)
  cell.dataset.column = String(address.column)
  cell.dataset.source = text
  if (alignment && alignment !== 'none') cell.style.textAlign = alignment
  // markdown-it escapes the text and renders no raw html.
  cell.innerHTML = renderInlineMarkdown(text)
  return cell
}

function listen(frame: HTMLElement, view: EditorView): void {
  frame.addEventListener('click', (event) => {
    const cell = cellOf(event.target)
    if (cell && !cell.isContentEditable) startEditing(cell)
  })
  frame.addEventListener('keydown', (event) => {
    const cell = cellOf(event.target)
    if (cell?.dataset.editing) handleEditingKey(event, cell, frame, view)
  })
  frame.addEventListener('focusout', (event) => {
    const cell = cellOf(event.target)
    if (cell?.dataset.editing) commit(cell, frame, view)
  })
  frame.addEventListener('contextmenu', (event) => {
    const cell = cellOf(event.target)
    if (!cell) return
    event.preventDefault()
    if (cell.dataset.editing) commit(cell, frame, view)
    frame.dispatchEvent(
      new CustomEvent<TableMenuRequest>(TABLE_MENU_EVENT, {
        bubbles: true,
        detail: menuRequest(event, cell, frame, view)
      })
    )
  })
}

function handleEditingKey(
  event: KeyboardEvent,
  cell: HTMLElement,
  frame: HTMLElement,
  view: EditorView
): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    stopEditing(cell, cell.dataset.source ?? '')
    return
  }
  const target = nextCell(event, addressOf(cell), frame)
  if (target === undefined) return
  event.preventDefault()
  commit(cell, frame, view)
  const next =
    target && frame.querySelector<HTMLElement>(`[data-row="${target.row}"][data-column="${target.column}"]`)
  if (next) startEditing(next)
}

/** Enter goes down a row, Tab and Shift-Tab across; null ends editing; undefined is not a move. */
function nextCell(
  event: KeyboardEvent,
  from: CellAddress,
  frame: HTMLElement
): CellAddress | null | undefined {
  const rows = Number(frame.dataset.rows)
  const columns = Number(frame.dataset.columns)
  const index = from.row * columns + from.column
  if (event.key === 'Enter') return from.row + 1 < rows ? { row: from.row + 1, column: from.column } : null
  if (event.key !== 'Tab') return undefined
  const target = index + (event.shiftKey ? -1 : 1)
  if (target < 0 || target >= rows * columns) return null
  return { row: Math.floor(target / columns), column: target % columns }
}

function startEditing(cell: HTMLElement): void {
  cell.dataset.editing = 'true'
  cell.contentEditable = 'plaintext-only'
  cell.textContent = cell.dataset.source ?? ''
  cell.focus()
  const selection = window.getSelection()
  selection?.selectAllChildren(cell)
  selection?.collapseToEnd()
}

function stopEditing(cell: HTMLElement, text: string): void {
  delete cell.dataset.editing
  cell.contentEditable = 'false'
  cell.innerHTML = renderInlineMarkdown(text)
}

/** Ends editing first, so the focus moving away cannot write the same cell twice. */
function commit(cell: HTMLElement, frame: HTMLElement, view: EditorView): void {
  const text = cell.textContent ?? ''
  stopEditing(cell, text)
  if (text.trim() === (cell.dataset.source ?? '')) return
  editTableCell(view, view.posAtDOM(frame), addressOf(cell), text)
}

function menuRequest(
  event: MouseEvent,
  cell: HTMLElement,
  frame: HTMLElement,
  view: EditorView
): TableMenuRequest {
  return {
    x: event.clientX,
    y: event.clientY,
    tablePosition: view.posAtDOM(frame),
    cell: addressOf(cell),
    rowCount: Number(frame.dataset.rows),
    columnCount: Number(frame.dataset.columns)
  }
}

function cellOf(target: EventTarget | null): HTMLElement | null {
  return target instanceof Element ? target.closest<HTMLElement>('th[data-row], td[data-row]') : null
}

function addressOf(cell: HTMLElement): CellAddress {
  return { row: Number(cell.dataset.row), column: Number(cell.dataset.column) }
}
