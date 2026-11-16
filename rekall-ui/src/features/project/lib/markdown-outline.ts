/** A heading of a markdown document, as the outline beside a component lists it. */
export interface OutlineHeading {
  readonly level: 2 | 3
  readonly text: string
}

const HEADING = /^(#{2,3})\s+(.+?)\s*#*\s*$/
const FENCE = /^\s*(```|~~~)/

/** The `##` and `###` headings of a document in the order written, leaving out anything inside a fenced block. */
export function outlineOf(markdown: string): OutlineHeading[] {
  const headings: OutlineHeading[] = []
  let inFence = false
  for (const line of markdown.split('\n')) {
    if (FENCE.test(line)) {
      inFence = !inFence
      continue
    }
    const found = inFence ? null : HEADING.exec(line)
    if (found) headings.push({ level: found[1]!.length === 2 ? 2 : 3, text: found[2]! })
  }
  return headings
}
