import { describe, expect, it } from 'vitest'
import { findTextRanges, matchOffsets } from '@/common/markdown/text-matches'

describe('matchOffsets', () => {
  it('finds every occurrence regardless of case', () => {
    expect(matchOffsets('Task task TASK', 'task')).toEqual([0, 5, 10])
  })

  it('does not overlap occurrences', () => {
    expect(matchOffsets('aaaa', 'aa')).toEqual([0, 2])
  })

  it('finds nothing for an empty query', () => {
    expect(matchOffsets('anything', '')).toEqual([])
  })
})

describe('findTextRanges', () => {
  function page(html: string): HTMLElement {
    const root = document.createElement('div')
    root.innerHTML = html
    return root
  }

  it('returns one range per occurrence, covering the matched text', () => {
    const ranges = findTextRanges(page('<p>one two</p><p>Two three</p>'), 'two')

    expect(ranges.map((range) => range.toString())).toEqual(['two', 'Two'])
  })

  it('matches across inline markup', () => {
    const ranges = findTextRanges(page('<p>foo <strong>bar</strong> baz</p>'), 'foo bar')

    expect(ranges.map((range) => range.toString())).toEqual(['foo bar'])
  })

  it('returns nothing when the text is absent or the page is empty', () => {
    expect(findTextRanges(page('<p>one</p>'), 'two')).toEqual([])
    expect(findTextRanges(page(''), 'two')).toEqual([])
  })
})
