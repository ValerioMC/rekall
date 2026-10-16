/**
 * Fits a title into at most `maxLines` lines of `maxWidth` pixels, measured by an estimate of
 * Fira Sans' advance widths rather than the DOM, so layout is pure and identical in tests.
 */

const NARROW = new Set('iljtf.,:;!|\'()[]{} I'.split(''))
const WIDE = new Set('mwMW@%'.split(''))

function charWidth(char: string, fontSize: number): number {
  if (NARROW.has(char)) return fontSize * 0.3
  if (WIDE.has(char)) return fontSize * 0.82
  if (char >= 'A' && char <= 'Z') return fontSize * 0.62
  if (char >= '0' && char <= '9') return fontSize * 0.55
  return fontSize * 0.51
}

export function textWidth(text: string, fontSize: number): number {
  let width = 0
  for (const char of text) width += charWidth(char, fontSize)
  return width
}

export function wrapText(text: string, maxWidth: number, fontSize: number, maxLines: number): string[] {
  const words = text.trim().split(/\s+/).filter((word) => word.length > 0)
  const lines: string[] = []
  let current = ''
  for (const word of words) {
    const candidate = current ? `${current} ${word}` : word
    if (textWidth(candidate, fontSize) <= maxWidth || !current) {
      current = candidate
      continue
    }
    lines.push(current)
    current = word
  }
  if (current) lines.push(current)
  if (lines.length <= maxLines && lines.every((line) => textWidth(line, fontSize) <= maxWidth)) {
    return lines
  }
  const kept = lines.slice(0, maxLines)
  const lastIndex = kept.length - 1
  const rest = lines.slice(lastIndex).join(' ')
  kept[lastIndex] = ellipsize(rest, maxWidth, fontSize)
  return kept.map((line, index) => (index < lastIndex ? ellipsize(line, maxWidth, fontSize) : line))
}

function ellipsize(text: string, maxWidth: number, fontSize: number): string {
  if (textWidth(text, fontSize) <= maxWidth) return text
  let cut = text
  while (cut.length > 1 && textWidth(`${cut}…`, fontSize) > maxWidth) cut = cut.slice(0, -1)
  return `${cut.trimEnd()}…`
}
