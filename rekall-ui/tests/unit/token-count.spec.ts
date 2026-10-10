import { describe, expect, it } from 'vitest'
import { describeTokens, formatTokens } from '@/common/format/token-count'

describe('formatTokens', () => {
  it('writes a small count as it is', () => {
    expect(formatTokens(0)).toBe('0')
    expect(formatTokens(950)).toBe('950')
  })

  it('keeps three significant figures at most past a thousand', () => {
    expect(formatTokens(1_000)).toBe('1k')
    expect(formatTokens(1_234)).toBe('1.23k')
    expect(formatTokens(12_400)).toBe('12.4k')
    expect(formatTokens(123_456)).toBe('123k')
    expect(formatTokens(3_100_000)).toBe('3.1M')
    expect(formatTokens(2_500_000_000)).toBe('2.5B')
  })

  it('never goes negative', () => {
    expect(formatTokens(-5)).toBe('0')
  })
})

describe('describeTokens', () => {
  it('puts what was generated first and what was read under it', () => {
    expect(describeTokens({ generated: 12_400, input: 30, cacheWrite: 5_000, cacheRead: 1_200_000 })).toBe(
      '12,400 tokens generated\n30 input · 5,000 cache write · 1,200,000 cache read'
    )
  })
})
