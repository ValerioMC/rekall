import { describe, expect, it } from 'vitest'
import { compactCount, tokensOf } from '@/model/context'
import { readableSize } from '@/model/backup'
import { isTextQuery } from '@/model/search'

describe('the context size figures', () => {
  it('shows a count under a thousand as it is, and thousands with one decimal at most', () => {
    expect(compactCount(842)).toBe('842')
    expect(compactCount(12_400)).toBe('12.4k')
    expect(compactCount(3_000)).toBe('3k')
    expect(compactCount(250_000)).toBe('250k')
  })

  it('estimates tokens the way the server does, rounding up', () => {
    expect(tokensOf(7)).toBe(2)
    expect(tokensOf(3500)).toBe(1000)
  })
})

describe('a backup size', () => {
  it('reads in the largest unit that keeps it above one', () => {
    expect(readableSize(512)).toBe('512 B')
    expect(readableSize(1536)).toBe('1.5 KB')
    expect(readableSize(42 * 1024 * 1024)).toBe('42 MB')
  })
})

describe('what the bar searches the text for', () => {
  it('a phrase of three characters or more', () => {
    expect(isTextQuery('settlement batch')).toBe(true)
    expect(isTextQuery('  ab ')).toBe(false)
  })

  it('never a query that is anchors, which the filter already resolves', () => {
    expect(isTextQuery('project:vega task:report')).toBe(false)
    expect(isTextQuery('task:report')).toBe(false)
  })
})
