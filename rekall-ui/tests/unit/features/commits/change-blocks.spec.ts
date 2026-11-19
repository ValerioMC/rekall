import { describe, expect, it } from 'vitest'
import { changeBlocks } from '@/features/commits/lib/change-blocks'

describe('changeBlocks', () => {
  it('splits five blocks between added and removed by their share of the changed lines', () => {
    expect(changeBlocks(3, 1)).toEqual(['added', 'added', 'added', 'added', 'removed'])
    expect(changeBlocks(1, 3)).toEqual(['added', 'removed', 'removed', 'removed', 'removed'])
  })

  it('keeps one block for a few additions beside many removals, and none for an untouched file', () => {
    expect(changeBlocks(1, 99)[0]).toBe('added')
    expect(changeBlocks(0, 0)).toEqual(['none', 'none', 'none', 'none', 'none'])
  })

  it('fills every block for a file only added', () => {
    expect(changeBlocks(40, 0).every((block) => block === 'added')).toBe(true)
  })
})
