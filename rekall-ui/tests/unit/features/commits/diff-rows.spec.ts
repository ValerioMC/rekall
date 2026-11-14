import { describe, expect, it } from 'vitest'
import { splitRows, unifiedRows } from '@/features/commits/lib/diff-rows'
import type { DiffHunk, DiffLine } from '@/features/commits/model/taskChanges'

function line(kind: DiffLine['kind'], oldNumber: number | null, newNumber: number | null, text: string): DiffLine {
  return { kind, oldNumber, newNumber, text }
}

const HUNK: DiffHunk = {
  oldStart: 1,
  oldLines: 3,
  newStart: 1,
  newLines: 4,
  context: '',
  lines: [
    line('CONTEXT', 1, 1, 'keep'),
    line('REMOVED', 2, null, 'pick: [session: Summary]'),
    line('ADDED', null, 2, "pick: [session: Summary, kind: 'task']"),
    line('ADDED', null, 3, 'extra'),
    line('CONTEXT', 3, 4, 'end')
  ]
}

describe('unifiedRows', () => {
  it('lists the removed lines before the added ones, with both numbers kept', () => {
    const rows = unifiedRows(HUNK, 'plain')

    expect(rows.map((row) => [row.kind, row.oldNumber, row.newNumber])).toEqual([
      ['CONTEXT', 1, 1],
      ['REMOVED', 2, null],
      ['ADDED', null, 2],
      ['ADDED', null, 3],
      ['CONTEXT', 3, 4]
    ])
  })

  it('marks the words an edited pair changed and leaves a lone added line whole', () => {
    const rows = unifiedRows(HUNK, 'plain')

    expect(rows[2]?.parts.some((part) => part.changed)).toBe(true)
    expect(rows[3]?.parts.every((part) => !part.changed)).toBe(true)
  })
})

describe('splitRows', () => {
  it('lays the removed line beside the first added one and leaves a blank beside the extra one', () => {
    const rows = splitRows(HUNK, 'plain')

    expect(rows).toHaveLength(4)
    expect(rows[1]?.left).toMatchObject({ kind: 'REMOVED', number: 2 })
    expect(rows[1]?.right).toMatchObject({ kind: 'ADDED', number: 2 })
    expect(rows[2]?.left.kind).toBe('EMPTY')
    expect(rows[2]?.right).toMatchObject({ kind: 'ADDED', number: 3 })
  })

  it('numbers a context line by the old file on the left and the new file on the right', () => {
    const rows = splitRows(HUNK, 'plain')

    expect(rows[3]?.left.number).toBe(3)
    expect(rows[3]?.right.number).toBe(4)
  })
})
