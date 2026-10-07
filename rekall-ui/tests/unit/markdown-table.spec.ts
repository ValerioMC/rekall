import { describe, expect, it } from 'vitest'
import {
  applyOperation,
  cellChange,
  escapeCellText,
  parseTable,
  serializeTable,
  toGrid,
  type TableChange
} from '@/common/live-preview/markdown-table'

const TABLE =
  '| Piece | What it does |\n|---|:---:|\n| `WrapupService` | writes the wrapup |\n| Clock | tells the time |'

function applied(source: string, change: TableChange): string {
  return source.slice(0, change.from) + change.insert + source.slice(change.to)
}

describe('parseTable', () => {
  it('reads the header, the alignments and the body rows', () => {
    const grid = toGrid(parseTable(TABLE))

    expect(grid.header).toEqual(['Piece', 'What it does'])
    expect(grid.alignments).toEqual(['none', 'center'])
    expect(grid.rows).toEqual([
      ['`WrapupService`', 'writes the wrapup'],
      ['Clock', 'tells the time']
    ])
  })

  it('places each cell exactly where its text sits in the markdown', () => {
    const table = parseTable(TABLE)

    for (const cell of [...table.header, ...table.rows.flat()]) {
      expect(TABLE.slice(cell.from, cell.to)).toBe(cell.text)
    }
  })

  it('reads a table written without its outer pipes', () => {
    const grid = toGrid(parseTable('a | b\n--- | ---:\n1 | 2'))

    expect(grid.header).toEqual(['a', 'b'])
    expect(grid.alignments).toEqual(['none', 'right'])
    expect(grid.rows).toEqual([['1', '2']])
  })

  it('keeps an escaped pipe inside its cell', () => {
    expect(toGrid(parseTable('| a \\| b | c |\n|---|---|')).header).toEqual(['a \\| b', 'c'])
  })

  it('pads a short row with empty cells', () => {
    expect(toGrid(parseTable('| a | b |\n|---|---|\n| 1 |')).rows).toEqual([['1', '']])
  })
})

describe('cellChange', () => {
  it('rewrites only the edited cell and leaves every other byte alone', () => {
    const change = cellChange(TABLE, { row: 2, column: 1 }, 'reads the clock')

    expect(applied(TABLE, change)).toBe(TABLE.replace('tells the time', 'reads the clock'))
  })

  it('edits a header cell', () => {
    expect(applied(TABLE, cellChange(TABLE, { row: 0, column: 0 }, 'Name'))).toBe(
      TABLE.replace('Piece', 'Name')
    )
  })

  it('fills an empty cell between its spaces', () => {
    const source = '| a | b |\n|---|---|\n|  | 2 |'

    expect(applied(source, cellChange(source, { row: 1, column: 0 }, 'one'))).toBe(
      '| a | b |\n|---|---|\n| one | 2 |'
    )
  })

  it('rewrites the table when the cell is missing from a short row', () => {
    const source = '| a | b |\n|---|---|\n| 1 |'

    expect(applied(source, cellChange(source, { row: 1, column: 1 }, '2'))).toBe(
      '| a | b |\n| --- | --- |\n| 1 | 2 |'
    )
  })

  it('escapes a pipe typed into a cell so it cannot split the row', () => {
    expect(cellChange(TABLE, { row: 1, column: 1 }, 'in | out').insert).toBe('in \\| out')
  })
})

describe('escapeCellText', () => {
  it('folds line breaks into spaces and trims, since a cell is one line', () => {
    expect(escapeCellText('  first\nsecond  ')).toBe('first second')
  })

  it('leaves a pipe that is already escaped as it is', () => {
    expect(escapeCellText('a \\| b')).toBe('a \\| b')
  })
})

describe('serializeTable', () => {
  it('writes the table compact, with no padding to the widest cell', () => {
    const grid = toGrid(parseTable('| Tool  | Version |\n|-------|---------|\n| Rust  | 1.85+   |'))

    expect(serializeTable(grid)).toBe('| Tool | Version |\n| --- | --- |\n| Rust | 1.85+ |')
  })

  it('writes each alignment back into the delimiter row', () => {
    const grid = {
      header: ['a', 'b', 'c', 'd'],
      alignments: ['none', 'left', 'center', 'right'] as const,
      rows: []
    }

    expect(serializeTable({ ...grid, alignments: [...grid.alignments] })).toBe(
      '| a | b | c | d |\n| --- | :--- | :---: | ---: |'
    )
  })
})

describe('applyOperation', () => {
  const grid = toGrid(parseTable('| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |'))

  it('inserts an empty body row at the position given', () => {
    expect(applyOperation(grid, { kind: 'insertRow', row: 1 }).rows).toEqual([
      ['1', '2'],
      ['', ''],
      ['3', '4']
    ])
  })

  it('deletes a body row', () => {
    expect(applyOperation(grid, { kind: 'deleteRow', row: 0 }).rows).toEqual([['3', '4']])
  })

  it('inserts an empty column in the header, the alignments and every row', () => {
    const wider = applyOperation(grid, { kind: 'insertColumn', column: 1 })

    expect(wider.header).toEqual(['a', '', 'b'])
    expect(wider.alignments).toEqual(['none', 'none', 'none'])
    expect(wider.rows).toEqual([
      ['1', '', '2'],
      ['3', '', '4']
    ])
  })

  it('deletes a column everywhere', () => {
    const narrower = applyOperation(grid, { kind: 'deleteColumn', column: 0 })

    expect(narrower.header).toEqual(['b'])
    expect(narrower.rows).toEqual([['2'], ['4']])
  })

  it('aligns one column and leaves the others', () => {
    expect(applyOperation(grid, { kind: 'align', column: 1, alignment: 'right' }).alignments).toEqual([
      'none',
      'right'
    ])
  })

  it('leaves the grid it was given untouched', () => {
    applyOperation(grid, { kind: 'deleteRow', row: 0 })

    expect(grid.rows).toHaveLength(2)
  })
})
