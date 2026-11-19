import { describe, expect, it } from 'vitest'
import { highlightSegments } from '@/features/search/lib/highlight'

describe('highlightSegments', () => {
  it('marks every part of the text that a term matches, whatever its case', () => {
    expect(highlightSegments('Fix Login timeout on login', ['login'])).toEqual([
      { text: 'Fix ', matched: false },
      { text: 'Login', matched: true },
      { text: ' timeout on ', matched: false },
      { text: 'login', matched: true }
    ])
  })

  it('leaves the text whole when there is no term worth marking', () => {
    expect(highlightSegments('Retry policy', [])).toEqual([{ text: 'Retry policy', matched: false }])
    expect(highlightSegments('Retry policy', ['r'])).toEqual([{ text: 'Retry policy', matched: false }])
  })

  it('treats a term as text, never as a pattern', () => {
    expect(highlightSegments('a (b) c', ['(b)'])).toEqual([
      { text: 'a ', matched: false },
      { text: '(b)', matched: true },
      { text: ' c', matched: false }
    ])
  })
})
