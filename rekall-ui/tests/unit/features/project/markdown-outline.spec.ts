import { describe, expect, it } from 'vitest'
import { outlineOf } from '@/features/project/lib/markdown-outline'

describe('the outline of a description', () => {
  it('lists the second and third level headings in the order written', () => {
    const markdown = '# Billing\n\n## Purpose\ntext\n### Why\n## Interfaces ##\n#### Too deep'

    expect(outlineOf(markdown)).toEqual([
      { level: 2, text: 'Purpose' },
      { level: 3, text: 'Why' },
      { level: 2, text: 'Interfaces' }
    ])
  })

  it('leaves out a heading that is inside a fenced block', () => {
    const markdown = '## Run\n```bash\n## not a heading\n```\n~~~\n## nor this\n~~~\n## Gotchas'

    expect(outlineOf(markdown).map((heading) => heading.text)).toEqual(['Run', 'Gotchas'])
  })

  it('is empty for text with no headings', () => {
    expect(outlineOf('Just prose.')).toEqual([])
  })
})
