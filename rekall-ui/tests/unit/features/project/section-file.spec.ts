import { describe, expect, it } from 'vitest'
import { sectionOfFile } from '@/features/project/lib/section-file'
import { SECTION_BODY_MAX_CHARACTERS } from '@/features/project/lib/section-size'

describe('a section made of a markdown file', () => {
  it('is titled by the first top-level heading and keeps the text whole', () => {
    const text = '# Platform reference\n\nHow it fits.\n## Overview\nx'

    expect(sectionOfFile('notes.md', text)).toEqual({ title: 'Platform reference', body: text })
  })

  it('is titled by the file name when no heading names it', () => {
    expect(sectionOfFile('glossary.md', 'Terms.').title).toBe('glossary')
  })

  it('ignores a heading inside a fenced block', () => {
    const text = '```\n# not a title\n```\ntext'

    expect(sectionOfFile('deploy.markdown', text).title).toBe('deploy')
  })

  it('refuses an empty file', () => {
    expect(() => sectionOfFile('empty.md', '  \n')).toThrow('empty.md is empty.')
  })

  it('refuses a file over the size a section holds, saying so', () => {
    const text = 'x'.repeat(SECTION_BODY_MAX_CHARACTERS + 1)

    expect(() => sectionOfFile('big.md', text)).toThrow(/100,001 characters; a section holds 100,000\. Nothing was imported\./)
  })

  it('takes a file of exactly the limit', () => {
    expect(sectionOfFile('edge.md', 'x'.repeat(SECTION_BODY_MAX_CHARACTERS)).body).toHaveLength(SECTION_BODY_MAX_CHARACTERS)
  })
})
