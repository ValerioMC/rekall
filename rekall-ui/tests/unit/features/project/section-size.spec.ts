import { describe, expect, it } from 'vitest'
import { compactCharacters, compactTokens, estimatedTokens } from '@/features/project/lib/section-size'

describe('the size of a section', () => {
  it('estimates a token for every four characters, rounding up', () => {
    expect(estimatedTokens(62_410)).toBe(15_603)
    expect(estimatedTokens(1)).toBe(1)
  })

  it('shows tokens and characters in the shortest readable form', () => {
    expect(compactTokens(950)).toBe('950')
    expect(compactTokens(15_603)).toBe('15.6k')
    expect(compactCharacters(3_180)).toBe('3,180')
    expect(compactCharacters(62_410)).toBe('62k')
  })
})
