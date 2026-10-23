export interface MatchPart {
  readonly text: string
  readonly hit: boolean
}

/** `text` cut around every occurrence of `needle`, ignoring case, so a filter can light what it matched. */
export function matchParts(text: string, needle: string): MatchPart[] {
  const wanted = needle.trim().toLowerCase()
  if (!wanted) return [{ text, hit: false }]
  const parts: MatchPart[] = []
  const lowered = text.toLowerCase()
  let from = 0
  for (let at = lowered.indexOf(wanted); at !== -1; at = lowered.indexOf(wanted, from)) {
    if (at > from) parts.push({ text: text.slice(from, at), hit: false })
    parts.push({ text: text.slice(at, at + wanted.length), hit: true })
    from = at + wanted.length
  }
  if (from < text.length) parts.push({ text: text.slice(from), hit: false })
  return parts
}
