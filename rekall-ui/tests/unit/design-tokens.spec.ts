import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'
import { describe, expect, it } from 'vitest'

/*
 * The design system lives in `main.css` as tokens. These rules keep components on them: a size,
 * corner or grey picked by hand is how the console drifted into twenty-two font sizes. The
 * terminal pane is exempt, because it frames a real terminal and is left as it is.
 */
const SOURCE = join(__dirname, '../../src')
const EXEMPT = new Set(['components/console/TerminalPane.vue'])

const RULES: ReadonlyArray<{ readonly name: string; readonly pattern: RegExp }> = [
  { name: 'a bracketed font size instead of text-micro…text-hero', pattern: /\btext-\[\d+(\.\d+)?px\]/ },
  { name: 'a bracketed radius instead of rounded-key, -control or -card', pattern: /\brounded(-[tblr]{1,2})?-\[([2-9]|\d{2,})px\]/ },
  { name: 'a radius token wrapped in var() instead of its utility', pattern: /\brounded(-[tblr]{1,2})?-\[var\(--radius-/ },
  { name: 'subtle text faded further with an opacity modifier', pattern: /\btext-text-subtle\/\d+/ },
  { name: 'a pixel font size in a style block instead of var(--text-…)', pattern: /font-size:\s*\d+(\.\d+)?px/ },
  { name: 'a numeric transition duration instead of a --motion tempo', pattern: /(?<=[\s"'`:])duration-\d+(?=[\s"'`])/ }
]

function sourceFiles(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry)
    if (statSync(path).isDirectory()) return sourceFiles(path)
    return /\.(vue|ts|css)$/.test(entry) ? [path] : []
  })
}

function violations(): string[] {
  const found: string[] = []
  for (const path of sourceFiles(SOURCE)) {
    const name = relative(SOURCE, path)
    if (EXEMPT.has(name)) continue
    readFileSync(path, 'utf8')
      .split('\n')
      .forEach((line, index) => {
        for (const rule of RULES) {
          if (rule.pattern.test(line)) found.push(`${name}:${index + 1} ${rule.name}`)
        }
      })
  }
  return found
}

describe('design tokens', () => {
  it('keeps every component on the type, radius, grey and motion tokens', () => {
    expect(violations()).toEqual([])
  })
})
