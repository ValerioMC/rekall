import { describe, expect, it } from 'vitest'
import { ref } from 'vue'
import { blankToNull, replaceById, trackSave, upsertById } from '@/stores/console/rows'

interface Row {
  readonly id: string
  readonly name: string
}

const rows: readonly Row[] = [
  { id: 'a', name: 'first' },
  { id: 'b', name: 'second' }
]

describe('replaceById', () => {
  it('swaps the row with the same id and keeps the order', () => {
    expect(replaceById(rows, { id: 'b', name: 'renamed' })).toEqual([
      { id: 'a', name: 'first' },
      { id: 'b', name: 'renamed' }
    ])
  })

  it('leaves a list without that id as it was', () => {
    expect(replaceById(rows, { id: 'z', name: 'stranger' })).toEqual(rows)
  })
})

describe('upsertById', () => {
  it('replaces a known row in place, wherever new rows would go', () => {
    expect(upsertById(rows, { id: 'a', name: 'renamed' }, 'last')[0]).toEqual({ id: 'a', name: 'renamed' })
  })

  it('puts an unknown row at the front when asked for first', () => {
    expect(upsertById(rows, { id: 'c', name: 'new' }, 'first').map((row) => row.id)).toEqual(['c', 'a', 'b'])
  })

  it('puts an unknown row at the end when asked for last', () => {
    expect(upsertById(rows, { id: 'c', name: 'new' }, 'last').map((row) => row.id)).toEqual(['a', 'b', 'c'])
  })
})

describe('blankToNull', () => {
  it('reads whitespace as none', () => {
    expect(blankToNull('  \n')).toBeNull()
  })

  it('keeps text as written, untrimmed', () => {
    expect(blankToNull(' text ')).toBe(' text ')
  })
})

describe('trackSave', () => {
  it('shows saving while the write runs, then saved, and hands back its result', async () => {
    const saveState = ref('saved')
    let during = ''
    const result = await trackSave(saveState, async () => {
      during = saveState.value
      return 42
    })
    expect(during).toBe('saving')
    expect(saveState.value).toBe('saved')
    expect(result).toBe(42)
  })

  it('marks a failed write unsaved and passes the failure on', async () => {
    const saveState = ref('saved')
    await expect(trackSave(saveState, () => Promise.reject(new Error('offline')))).rejects.toThrow('offline')
    expect(saveState.value).toBe('unsaved')
  })
})
