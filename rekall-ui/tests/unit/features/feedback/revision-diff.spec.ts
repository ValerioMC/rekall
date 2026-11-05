import { describe, expect, it } from 'vitest'
import { diffLines, foldUnchanged } from '@/features/feedback/lib/revision-diff'

describe('diffLines', () => {
  it('keeps a text that did not change as one run of kept lines', () => {
    expect(diffLines('a\nb', 'a\nb')).toEqual([{ kind: 'same', lines: ['a', 'b'] }])
  })

  it('shows an insertion between kept lines', () => {
    expect(diffLines('a\nc', 'a\nb\nc')).toEqual([
      { kind: 'same', lines: ['a'] },
      { kind: 'added', lines: ['b'] },
      { kind: 'same', lines: ['c'] }
    ])
  })

  it('shows a deletion between kept lines', () => {
    expect(diffLines('a\nb\nc', 'a\nc')).toEqual([
      { kind: 'same', lines: ['a'] },
      { kind: 'removed', lines: ['b'] },
      { kind: 'same', lines: ['c'] }
    ])
  })

  it('shows a rewritten line as the old one removed and the new one added', () => {
    expect(diffLines('Export to CSV.', 'Export to CSV and XLSX.')).toEqual([
      { kind: 'removed', lines: ['Export to CSV.'] },
      { kind: 'added', lines: ['Export to CSV and XLSX.'] }
    ])
  })

  it('groups consecutive lines of the same kind', () => {
    const hunks = diffLines('keep', 'keep\nnew one\nnew two')
    expect(hunks[1]).toEqual({ kind: 'added', lines: ['new one', 'new two'] })
  })
})

describe('foldUnchanged', () => {
  const run = (count: number): string => Array.from({ length: count }, (_, index) => `line ${index}`).join('\n')

  it('leaves a short kept run as it is', () => {
    expect(foldUnchanged(diffLines(run(3), run(3)))).toEqual([{ kind: 'same', lines: ['line 0', 'line 1', 'line 2'] }])
  })

  it('folds the middle of a long kept run and keeps a line of context beside each change', () => {
    const rows = foldUnchanged(diffLines(`${run(8)}\nold`, `${run(8)}\nnew`))

    expect(rows.map((row) => row.kind)).toEqual(['folded', 'same', 'removed', 'added'])
    expect(rows[0]?.lines).toHaveLength(7)
    expect(rows[1]?.lines).toEqual(['line 7'])
  })
})
