import { describe, expect, it } from 'vitest'
import { renderMarkdown } from '@/common/utils/markdown'

describe('renderMarkdown', () => {
  it('renders headings, lists and inline code', () => {
    const html = renderMarkdown('# Contesto\n\n- primo\n- secondo\n\nUsa `kubectl` qui.')

    expect(html).toContain('<h1')
    expect(html).toContain('Contesto')
    expect(html).toContain('<li>primo</li>')
    expect(html).toContain('<code')
    expect(html).toContain('kubectl')
  })

  it('escapes html so a document cannot inject markup', () => {
    const html = renderMarkdown('<img src=x onerror="alert(1)">')

    expect(html).not.toContain('<img')
    expect(html).toContain('&lt;img')
  })

  it('keeps fenced code verbatim and unescaped as text', () => {
    const html = renderMarkdown('```\nSELECT * FROM t WHERE a < b\n```')

    expect(html).toContain('<pre')
    expect(html).toContain('SELECT * FROM t WHERE a &lt; b')
  })

  it('renders links with a safe target', () => {
    const html = renderMarkdown('[gitlab](https://gitlab.example.org)')

    expect(html).toContain('rel="noreferrer"')
    expect(html).toContain('href="https://gitlab.example.org"')
  })

  it('returns an empty string for empty input', () => {
    expect(renderMarkdown('')).toBe('')
  })
})
