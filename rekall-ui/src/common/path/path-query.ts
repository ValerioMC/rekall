import type { DirectoryEntry } from '@/model/filesystem'

/**
 * What the path picker's filter field holds, split the way a shell splits a half-typed path:
 * everything up to the last slash names a folder to list, and what follows filters that
 * folder's entries by name. `src/comp` lists `src` and keeps what matches `comp`; `~/` lists
 * the home folder; `comp` filters the folder the picker is already in.
 */
export interface PathQuery {
  /** Resolved by the server against the current folder; empty means the current folder itself. */
  readonly folder: string
  readonly needle: string
}

/** Typed on their own, these name a folder rather than filter one: nothing is called `~` or `..`. */
const FOLDER_WORDS = new Set(['~', '..'])

export function splitPathQuery(query: string): PathQuery {
  if (FOLDER_WORDS.has(query)) return { folder: query, needle: '' }
  const cut = query.lastIndexOf('/')
  if (cut < 0) return { folder: '', needle: query }
  return { folder: query.slice(0, cut + 1), needle: query.slice(cut + 1) }
}

/** A half-open `[start, end)` run of characters in an entry's name that the needle matched. */
export type MatchRange = readonly [start: number, end: number]

export interface PathMatch {
  /** Lower is better: a prefix beats a substring, which beats letters scattered through the name. */
  readonly score: number
  readonly ranges: readonly MatchRange[]
}

export interface PathRow {
  readonly entry: DirectoryEntry
  readonly ranges: readonly MatchRange[]
}

const PREFIX = 0
const SUBSTRING = 100
const SCATTERED = 1000

/**
 * Matches a needle against a name ignoring case: as a prefix, then anywhere as one run, then
 * as letters in order with gaps (`mdedt` finds `markdown-editor.ts`). Null when none of the
 * three holds.
 */
export function matchName(name: string, needle: string): PathMatch | null {
  if (!needle) return { score: PREFIX, ranges: [] }
  const hay = name.toLowerCase()
  const want = needle.toLowerCase()

  const at = hay.indexOf(want)
  if (at === 0) return { score: PREFIX, ranges: [[0, want.length]] }
  if (at > 0) return { score: SUBSTRING + at, ranges: [[at, at + want.length]] }

  const ranges: [number, number][] = []
  let from = 0
  let gaps = 0
  for (const letter of want) {
    const found = hay.indexOf(letter, from)
    if (found < 0) return null
    const last = ranges[ranges.length - 1]
    if (last && last[1] === found) {
      last[1] = found + 1
    } else {
      if (ranges.length) gaps += found - from
      ranges.push([found, found + 1])
    }
    from = found + 1
  }
  return { score: SCATTERED + gaps * 10 + ranges.length, ranges }
}

/**
 * The rows the picker shows. With no needle it keeps the server's order (folders first, then
 * by name). With one, the best match leads and a folder wins a tie. Hidden entries stay out
 * unless they are asked for, or the needle itself starts with a dot, which is how anyone
 * looking for `.env` would start typing.
 */
export function filterEntries(
  entries: readonly DirectoryEntry[],
  needle: string,
  showHidden: boolean
): PathRow[] {
  const includeHidden = showHidden || needle.startsWith('.')
  const visible = entries.filter((entry) => includeHidden || !entry.hidden)
  if (!needle) return visible.map((entry) => ({ entry, ranges: [] }))

  const matched: { row: PathRow; score: number; order: number }[] = []
  visible.forEach((entry, order) => {
    const match = matchName(entry.name, needle)
    if (match) matched.push({ row: { entry, ranges: match.ranges }, score: match.score, order })
  })
  matched.sort(
    (a, b) =>
      a.score - b.score ||
      Number(b.row.entry.directory) - Number(a.row.entry.directory) ||
      a.order - b.order
  )
  return matched.map((item) => item.row)
}

/** A name cut into the runs the needle matched and the runs between them, for highlighting. */
export function splitByRanges(
  name: string,
  ranges: readonly MatchRange[]
): { readonly text: string; readonly matched: boolean }[] {
  const parts: { text: string; matched: boolean }[] = []
  let cursor = 0
  for (const [start, end] of ranges) {
    if (start > cursor) parts.push({ text: name.slice(cursor, start), matched: false })
    parts.push({ text: name.slice(start, end), matched: true })
    cursor = end
  }
  if (cursor < name.length) parts.push({ text: name.slice(cursor), matched: false })
  return parts
}

/**
 * A folder's path ends in its separator, so whoever reads the markdown (a person or a session)
 * can tell `src/` from a file called `src` without looking at the disk.
 */
export function withFolderSlash(path: string, directory: boolean): string {
  if (!directory || /[\\/]$/.test(path)) return path
  const separator = path.includes('\\') && !path.includes('/') ? '\\' : '/'
  return `${path}${separator}`
}

/**
 * True when the text before the caret leaves it inside code: an unclosed fence (``` or ~~~ at
 * the start of a line), or an odd number of backticks on the current line.
 */
export function isInsideCode(textBefore: string): boolean {
  const fences = textBefore.split('\n').filter((line) => /^ {0,3}(```|~~~)/.test(line)).length
  if (fences % 2 === 1) return true
  const line = textBefore.slice(textBefore.lastIndexOf('\n') + 1)
  const ticks = line.replace(/\\`/g, '').split('`').length - 1
  return ticks % 2 === 1
}

/**
 * The text that lands in the markdown for a picked path.
 *
 * Wrapped as inline code, because a raw path is not safe prose: `_` and `*` in a folder name turn
 * into emphasis in the preview and the path silently loses characters. Inside code already (a
 * fence, or between backticks the writer opened) it goes in bare, since a second pair of
 * backticks would close the code instead. A path holding a backtick of its own gets the
 * double-backtick form, which CommonMark reads with the padding stripped.
 */
export function formatPathForMarkdown(path: string, directory: boolean, textBefore: string): string {
  const full = withFolderSlash(path, directory)
  if (isInsideCode(textBefore)) return full
  return full.includes('`') ? `\`\` ${full} \`\`` : `\`${full}\``
}
