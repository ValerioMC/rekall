import type { config } from 'md-editor-v3'

type MarkdownIt = Parameters<NonNullable<Parameters<typeof config>[0]['markdownItConfig']>>[0]
type RenderRule = NonNullable<MarkdownIt['renderer']['rules'][string]>

export const TABLE_FRAME_CLASS = 'rk-table-frame'
export const SHORT_CELL_CLASS = 'rk-short-cell'
const SHORT_CELL_LIMIT = 30

/** Renders every table inside a frame, so a read view scrolls a wide one instead of letting it run past the page. */
export function markTableFrames(md: MarkdownIt): void {
  const open = ruleOrDefault(md, 'table_open')
  const close = ruleOrDefault(md, 'table_close')
  md.renderer.rules['table_open'] = (...token) => `<div class="${TABLE_FRAME_CLASS}">${open(...token)}`
  md.renderer.rules['table_close'] = (...token) => `${close(...token)}</div>`
}

function ruleOrDefault(md: MarkdownIt, name: string): RenderRule {
  return md.renderer.rules[name] ?? ((tokens, index, options, _env, renderer) => renderer.renderToken(tokens, index, options))
}

/** A column whose every cell is short never wraps: its width comes out of the columns that hold prose. */
export function keepShortColumnsWhole(table: HTMLTableElement): void {
  const rows = Array.from(table.rows)
  const columnCount = Math.max(0, ...rows.map((row) => row.cells.length))
  for (let column = 0; column < columnCount; column++) {
    const cells = rows.flatMap((row) => row.cells[column] ?? [])
    const isShort = cells.every(isShortCell)
    cells.forEach((cell) => cell.classList.toggle(SHORT_CELL_CLASS, isShort))
  }
}

function isShortCell(cell: HTMLTableCellElement): boolean {
  return (cell.textContent ?? '').trim().length <= SHORT_CELL_LIMIT
}
