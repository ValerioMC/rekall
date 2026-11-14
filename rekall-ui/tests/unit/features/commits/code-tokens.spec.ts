import { describe, expect, it } from 'vitest'
import { codeParts, languageOf } from '@/features/commits/lib/code-tokens'

describe('languageOf', () => {
  it('reads the comment style from the extension', () => {
    expect(languageOf('src/App.vue')).toBe('slash')
    expect(languageOf('scripts/run.py')).toBe('hash')
    expect(languageOf('docs/guide.md')).toBe('plain')
    expect(languageOf('Makefile')).toBe('plain')
  })
})

describe('codeParts', () => {
  it('tones keywords, strings and a trailing comment', () => {
    const parts = codeParts("const name = 'x' // why", 'slash', null)

    expect(parts.map((part) => [part.text, part.tone])).toEqual([
      ['const', 'keyword'],
      [' name = ', 'plain'],
      ["'x'", 'string'],
      [' ', 'plain'],
      ['// why', 'comment']
    ])
  })

  it('keeps plain text whole and carries the changed flag of each segment', () => {
    const parts = codeParts('one two', 'plain', [
      { text: 'one ', changed: false },
      { text: 'two', changed: true }
    ])

    expect(parts).toEqual([
      { text: 'one ', changed: false, tone: 'plain' },
      { text: 'two', changed: true, tone: 'plain' }
    ])
  })

  it('does not tone a word that only contains a keyword', () => {
    expect(codeParts('constant', 'slash', null).every((part) => part.tone === 'plain')).toBe(true)
  })
})
