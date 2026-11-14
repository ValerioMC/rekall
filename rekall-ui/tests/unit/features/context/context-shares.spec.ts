import { describe, expect, it } from 'vitest'
import { contextShares } from '@/features/context/lib/context-shares'
import type { ContextSize } from '@/features/context/model/context'

function sizeOf(...characters: number[]): ContextSize {
  const parts = characters.map((count, index) => ({ label: `Part ${index}`, characters: count, reference: false }))
  return { characters: characters.reduce((sum, count) => sum + count, 0), estimatedTokens: 0, parts }
}

describe('contextShares', () => {
  it('orders the parts from the heaviest and gives each its share', () => {
    const shares = contextShares(sizeOf(100, 300), 5)

    expect(shares.map((share) => share.label)).toEqual(['Part 1', 'Part 0'])
    expect(shares.map((share) => share.percent)).toEqual([75, 25])
  })

  it('folds the lightest parts into one row when there are more than the limit', () => {
    const shares = contextShares(sizeOf(400, 300, 200, 100), 3)

    expect(shares.map((share) => share.label)).toEqual(['Part 0', 'Part 1', 'Others'])
    expect(shares[2]!.percent).toBe(30)
  })

  it('keeps a reference note marked', () => {
    const size: ContextSize = { characters: 10, estimatedTokens: 3, parts: [{ label: 'Note: a.md', characters: 10, reference: true }] }

    expect(contextShares(size, 5)[0]!.reference).toBe(true)
  })

  it('has no rows for an empty context', () => {
    expect(contextShares(sizeOf(), 5)).toEqual([])
  })
})
