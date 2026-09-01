/** How much of a body a card shows before it is cut. Two lines at the sizes the cards use. */
const DEFAULT_LIMIT = 110

/**
 * A couple of lines of a markdown body, as prose.
 *
 * The markup is stripped rather than rendered — a preview is read at a glance, and `##` and
 * `**` at that size are noise — and the cut lands on a word rather than in the middle of one,
 * because a card that ends "e lo pubblic" reads as a bug in the text rather than as a preview.
 */
export function excerpt(body: string, limit: number = DEFAULT_LIMIT): string {
  const flat = body
    .replace(/[#`>*|_-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
  if (flat.length <= limit) return flat

  const cut = flat.slice(0, limit)
  const lastSpace = cut.lastIndexOf(' ')
  // A word longer than a third of the limit is cut where it falls: better a broken word than
  // a preview that gives back almost nothing.
  const kept = lastSpace > limit * 0.66 ? cut.slice(0, lastSpace) : cut
  return `${kept.trimEnd()}…`
}
