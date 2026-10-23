import { describe, expect, it } from 'vitest'
import { matchParts } from '@/common/format/match-parts'

describe('match parts', () => {
  it('cuts the text around every occurrence, ignoring case and keeping the original spelling', () => {
    expect(matchParts('@Babel/babel-core', 'BABEL')).toEqual([
      { text: '@', hit: false },
      { text: 'Babel', hit: true },
      { text: '/', hit: false },
      { text: 'babel', hit: true },
      { text: '-core', hit: false }
    ])
  })

  it('leaves the text whole when there is nothing to find', () => {
    expect(matchParts('serde', '  ')).toEqual([{ text: 'serde', hit: false }])
    expect(matchParts('serde', 'tokio')).toEqual([{ text: 'serde', hit: false }])
  })
})
