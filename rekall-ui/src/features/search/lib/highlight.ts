export interface TextSegment {
  readonly text: string
  readonly matched: boolean
}

function escaped(term: string): string {
  return term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/** `text` cut into the parts that match a query term and the parts that do not, for rendering without markup. */
export function highlightSegments(text: string, terms: readonly string[]): readonly TextSegment[] {
  const usable = terms.filter((term) => term.length > 1)
  if (usable.length === 0 || text.length === 0) return [{ text, matched: false }]
  const pattern = new RegExp(`(${usable.map(escaped).join('|')})`, 'gi')
  return text
    .split(pattern)
    .filter((part) => part.length > 0)
    .map((part) => ({ text: part, matched: usable.some((term) => part.toLowerCase() === term.toLowerCase()) }))
}
