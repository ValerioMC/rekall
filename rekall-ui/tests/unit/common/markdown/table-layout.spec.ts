import MarkdownIt from 'markdown-it'
import { describe, expect, it } from 'vitest'
import { keepShortColumnsWhole, markTableFrames, SHORT_CELL_CLASS, TABLE_FRAME_CLASS } from '@/common/markdown/table-layout'

function renderTable(markdown: string): HTMLTableElement {
  const renderer = new MarkdownIt()
  markTableFrames(renderer)
  const host = document.createElement('div')
  host.innerHTML = renderer.render(markdown)
  return host.querySelector('table')!
}

function shortColumns(table: HTMLTableElement): boolean[][] {
  return Array.from(table.rows, (row) => Array.from(row.cells, (cell) => cell.classList.contains(SHORT_CELL_CLASS)))
}

describe('markTableFrames', () => {
  it('renders each table inside its own frame', () => {
    const renderer = new MarkdownIt()
    markTableFrames(renderer)
    const host = document.createElement('div')
    host.innerHTML = renderer.render('| a |\n|---|\n| 1 |\n\nBetween.\n\n| b |\n|---|\n| 2 |')

    expect(host.querySelectorAll(`.${TABLE_FRAME_CLASS} > table`)).toHaveLength(2)
    expect(host.querySelector(`.${TABLE_FRAME_CLASS} p`)).toBeNull()
  })
})

describe('keepShortColumnsWhole', () => {
  it('marks every cell of a column whose cells are all short, and none of a column with prose', () => {
    const table = renderTable('| Area | What it provides |\n|---|---|\n| **Operational context** | The context needed to tell a normal run apart |')

    keepShortColumnsWhole(table)

    expect(shortColumns(table)).toEqual([
      [true, false],
      [true, false]
    ])
  })

  it('counts the rendered text, up to thirty characters', () => {
    const thirty = 'a'.repeat(30)
    const table = renderTable(`| at the limit | past it |\n|---|---|\n| **\`${thirty}\`** | ${thirty}b |`)

    keepShortColumnsWhole(table)

    expect(shortColumns(table)).toEqual([
      [true, false],
      [true, false]
    ])
  })

  it('lets a column wrap again once one of its cells grows long', () => {
    const table = renderTable('| Area |\n|---|\n| Clock |')
    keepShortColumnsWhole(table)

    table.rows[1]!.cells[0]!.textContent = 'A description that is far longer than a label'
    keepShortColumnsWhole(table)

    expect(shortColumns(table)).toEqual([[false], [false]])
  })
})
