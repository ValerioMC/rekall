export interface TextSegment {
  readonly text: string
  readonly changed: boolean
}

export interface WordDiff {
  readonly removed: readonly TextSegment[]
  readonly added: readonly TextSegment[]
}

const SIMILAR_SHARE = 0.4

/** Marks the part of two lines that differs, when they share enough to be one line edited; otherwise null. */
export function wordDiff(removed: string, added: string): WordDiff | null {
  const prefix = commonPrefix(removed, added)
  const suffix = commonSuffix(removed.slice(prefix), added.slice(prefix))
  const shortest = Math.min(removed.length, added.length)
  if (shortest === 0 || prefix + suffix < SIMILAR_SHARE * shortest) return null
  return { removed: segmentsOf(removed, prefix, suffix), added: segmentsOf(added, prefix, suffix) }
}

function commonPrefix(a: string, b: string): number {
  let length = 0
  while (length < a.length && length < b.length && a[length] === b[length]) length++
  return length
}

function commonSuffix(a: string, b: string): number {
  let length = 0
  while (length < a.length && length < b.length && a[a.length - 1 - length] === b[b.length - 1 - length]) length++
  return length
}

function segmentsOf(text: string, prefix: number, suffix: number): readonly TextSegment[] {
  const middleEnd = text.length - suffix
  const parts: TextSegment[] = [
    { text: text.slice(0, prefix), changed: false },
    { text: text.slice(prefix, middleEnd), changed: true },
    { text: text.slice(middleEnd), changed: false }
  ]
  return parts.filter((part) => part.text.length > 0)
}
