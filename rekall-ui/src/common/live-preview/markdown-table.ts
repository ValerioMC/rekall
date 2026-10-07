/**
 * A GFM table read from its markdown and written back. The markdown is what is stored, so a cell
 * edit replaces only that cell's text, and a structural edit (rows, columns, alignment) rewrites
 * only the table it touched, compact and unpadded so a table costs no extra tokens.
 */

export type ColumnAlignment = 'none' | 'left' | 'center' | 'right'

/** One cell's text and where it sits, as offsets into the table's own markdown. */
export interface SourceCell {
  text: string
  from: number
  to: number
}

export interface ParsedTable {
  header: SourceCell[]
  alignments: ColumnAlignment[]
  rows: SourceCell[][]
}

/** Row 0 is the header; row 1 is the first body row. */
export interface CellAddress {
  row: number
  column: number
}

export interface TableGrid {
  header: string[]
  alignments: ColumnAlignment[]
  rows: string[][]
}

/** Rows here count body rows only: the header is never inserted or deleted. */
export type TableOperation =
  | { kind: 'insertRow'; row: number }
  | { kind: 'deleteRow'; row: number }
  | { kind: 'insertColumn'; column: number }
  | { kind: 'deleteColumn'; column: number }
  | { kind: 'align'; column: number; alignment: ColumnAlignment }

/** Replace `from`..`to` of the table's markdown with `insert`. */
export interface TableChange {
  from: number
  to: number
  insert: string
}

export function parseTable(source: string): ParsedTable {
  const lines = linesWithOffsets(source)
  const header = splitRow(lines[0]?.text ?? '', lines[0]?.offset ?? 0)
  const alignments = splitRow(lines[1]?.text ?? '', 0).map((cell) => alignmentOf(cell.text))
  const rows = lines.slice(2).map((line) => splitRow(line.text, line.offset))
  return { header, alignments, rows }
}

export function toGrid(table: ParsedTable): TableGrid {
  const width = table.header.length
  return {
    header: table.header.map((cell) => cell.text),
    alignments: fit(table.alignments, width, 'none'),
    rows: table.rows.map((row) =>
      fit(
        row.map((cell) => cell.text),
        width,
        ''
      )
    )
  }
}

export function serializeTable(grid: TableGrid): string {
  const width = grid.header.length
  const delimiter = fit(grid.alignments, width, 'none').map(delimiterOf)
  return [grid.header, delimiter, ...grid.rows.map((row) => fit(row, width, ''))].map(rowLine).join('\n')
}

export function applyOperation(grid: TableGrid, operation: TableOperation): TableGrid {
  switch (operation.kind) {
    case 'insertRow':
      return {
        ...grid,
        rows: spliced(
          grid.rows,
          operation.row,
          0,
          grid.header.map(() => '')
        )
      }
    case 'deleteRow':
      return { ...grid, rows: spliced(grid.rows, operation.row, 1) }
    case 'insertColumn':
      return {
        header: spliced(grid.header, operation.column, 0, ''),
        alignments: spliced(grid.alignments, operation.column, 0, 'none' as ColumnAlignment),
        rows: grid.rows.map((row) => spliced(row, operation.column, 0, ''))
      }
    case 'deleteColumn':
      return {
        header: spliced(grid.header, operation.column, 1),
        alignments: spliced(grid.alignments, operation.column, 1),
        rows: grid.rows.map((row) => spliced(row, operation.column, 1))
      }
    case 'align':
      return {
        ...grid,
        alignments: grid.alignments.map((current, index) =>
          index === operation.column ? operation.alignment : current
        )
      }
  }
}

/**
 * The change that puts `text` in one cell. A cell the row's markdown never wrote (a short row)
 * has no range to replace, so the whole table is rewritten instead.
 */
export function cellChange(source: string, address: CellAddress, text: string): TableChange {
  const table = parseTable(source)
  const row = address.row === 0 ? table.header : table.rows[address.row - 1]
  const cell = row?.[address.column]
  const escaped = escapeCellText(text)
  if (cell) return { from: cell.from, to: cell.to, insert: escaped }
  const grid = toGrid(table)
  const rows = grid.rows.map((cells, index) =>
    index === address.row - 1
      ? cells.map((current, column) => (column === address.column ? escaped : current))
      : cells
  )
  return { from: 0, to: source.length, insert: serializeTable({ ...grid, rows }) }
}

/** A cell is one line, and a bare pipe would end it early. */
export function escapeCellText(text: string): string {
  return text
    .replace(/\r?\n/g, ' ')
    .replace(/(^|[^\\])\|/g, '$1\\|')
    .trim()
}

function linesWithOffsets(source: string): { text: string; offset: number }[] {
  let offset = 0
  return source.split('\n').map((text) => {
    const line = { text, offset }
    offset += text.length + 1
    return line
  })
}

/** Splits on unescaped pipes; the outer pipes are optional in GFM and frame no cell. */
function splitRow(line: string, offset: number): SourceCell[] {
  const boundaries = unescapedPipes(line)
  const first = line.search(/\S/)
  const last = line.trimEnd().length - 1
  const opens = boundaries[0] === first
  const closes = boundaries.length > (opens ? 1 : 0) && boundaries[boundaries.length - 1] === last
  const inner = boundaries.slice(opens ? 1 : 0, closes ? -1 : undefined)
  const start = opens ? first + 1 : Math.max(first, 0)
  const end = closes ? last : line.length
  const edges = [start - 1, ...inner, end]
  const cells: SourceCell[] = []
  for (let index = 0; index < edges.length - 1; index++) {
    cells.push(trimmedCell(line, edges[index]! + 1, edges[index + 1]!, offset))
  }
  return cells
}

function unescapedPipes(line: string): number[] {
  const positions: number[] = []
  for (let index = 0; index < line.length; index++) {
    if (line[index] === '\\') index++
    else if (line[index] === '|') positions.push(index)
  }
  return positions
}

/** An empty cell's range sits after its first space, so typing into it keeps `| x |` spacing. */
function trimmedCell(line: string, from: number, to: number, offset: number): SourceCell {
  const segment = line.slice(from, to)
  const text = segment.trim()
  if (!text) {
    const at = from + Math.min(segment.length, 1)
    return { text: '', from: offset + at, to: offset + at }
  }
  const lead = segment.length - segment.trimStart().length
  return { text, from: offset + from + lead, to: offset + from + lead + text.length }
}

function alignmentOf(delimiter: string): ColumnAlignment {
  const left = delimiter.startsWith(':')
  const right = delimiter.endsWith(':')
  if (left && right) return 'center'
  if (left) return 'left'
  if (right) return 'right'
  return 'none'
}

function delimiterOf(alignment: ColumnAlignment): string {
  return { none: '---', left: ':---', center: ':---:', right: '---:' }[alignment]
}

function rowLine(cells: string[]): string {
  return `| ${cells.join(' | ')} |`
}

function fit<T>(values: T[], width: number, filler: T): T[] {
  return Array.from({ length: width }, (_, index) => values[index] ?? filler)
}

function spliced<T>(values: T[], at: number, remove: number, ...insert: T[]): T[] {
  const copy = [...values]
  copy.splice(at, remove, ...insert)
  return copy
}
