/** Where each occurrence of `query` starts in `text`, ignoring case; occurrences never overlap. */
export function matchOffsets(text: string, query: string): number[] {
  if (!query) return []
  const haystack = text.toLowerCase()
  const needle = query.toLowerCase()
  const offsets: number[] = []
  for (let at = haystack.indexOf(needle); at !== -1; at = haystack.indexOf(needle, at + needle.length)) {
    offsets.push(at)
  }
  return offsets
}

interface TextSlice {
  node: Text
  start: number
}

function collectTextSlices(root: Node): { slices: TextSlice[]; text: string } {
  const slices: TextSlice[] = []
  let text = ''
  const visit = (node: Node): void => {
    if (node.nodeType === Node.TEXT_NODE) {
      slices.push({ node: node as Text, start: text.length })
      text += node.nodeValue ?? ''
      return
    }
    node.childNodes.forEach(visit)
  }
  visit(root)
  return { slices, text }
}

function pointAt(
  slices: TextSlice[],
  last: TextSlice,
  offset: number,
  isEnd: boolean
): { node: Text; offset: number } {
  const reaches = (slice: TextSlice): boolean => {
    const end = slice.start + slice.node.length
    return offset >= slice.start && (isEnd ? offset <= end : offset < end)
  }
  const slice = slices.find(reaches) ?? last
  return { node: slice.node, offset: offset - slice.start }
}

/**
 * One range per occurrence of `query` in the text under `root`. The text is read as one string, so
 * an occurrence may cross inline markup (`foo **bar**` matches "foo bar").
 */
export function findTextRanges(root: Node, query: string): Range[] {
  const { slices, text } = collectTextSlices(root)
  const last = slices[slices.length - 1]
  if (!last) return []
  return matchOffsets(text, query).map((offset) => {
    const from = pointAt(slices, last, offset, false)
    const to = pointAt(slices, last, offset + query.length, true)
    const range = document.createRange()
    range.setStart(from.node, from.offset)
    range.setEnd(to.node, to.offset)
    return range
  })
}
