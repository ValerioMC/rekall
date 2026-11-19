import { codeParts, type CodeLanguage, type CodePart } from '@/features/commits/lib/code-tokens'
import { wordDiff, type TextSegment } from '@/features/commits/lib/word-diff'
import type { DiffHunk, DiffLine, DiffLineKind } from '@/features/commits/model/taskChanges'

export interface UnifiedRow {
  readonly kind: DiffLineKind
  readonly oldNumber: number | null
  readonly newNumber: number | null
  readonly parts: readonly CodePart[]
}

/** One side of a split row; `EMPTY` is the blank beside a line that exists on the other side only. */
export interface SplitCell {
  readonly kind: DiffLineKind | 'EMPTY'
  readonly number: number | null
  readonly parts: readonly CodePart[]
}

export interface SplitRow {
  readonly left: SplitCell
  readonly right: SplitCell
}

type Item =
  | { readonly kind: 'context'; readonly line: DiffLine }
  | { readonly kind: 'run'; readonly removed: readonly DiffLine[]; readonly added: readonly DiffLine[] }

const EMPTY_CELL: SplitCell = { kind: 'EMPTY', number: null, parts: [] }

/** Removed lines then added lines, one row each, with the words an edit changed marked. */
export function unifiedRows(hunk: DiffHunk, language: CodeLanguage): readonly UnifiedRow[] {
  return groupItems(hunk.lines).flatMap((item) => {
    if (item.kind === 'context') return [unifiedRow(item.line, language, null)]
    const pairs = pairedSegments(item.removed, item.added)
    return [
      ...item.removed.map((line, index) => unifiedRow(line, language, pairs[index]?.removed ?? null)),
      ...item.added.map((line, index) => unifiedRow(line, language, pairs[index]?.added ?? null))
    ]
  })
}

/** Old file on the left, new on the right; a run of removed and added lines is laid side by side. */
export function splitRows(hunk: DiffHunk, language: CodeLanguage): readonly SplitRow[] {
  return groupItems(hunk.lines).flatMap((item) => {
    if (item.kind === 'context') {
      return [{ left: cell(item.line, language, null, item.line.oldNumber), right: cell(item.line, language, null, item.line.newNumber) }]
    }
    const pairs = pairedSegments(item.removed, item.added)
    const height = Math.max(item.removed.length, item.added.length)
    return Array.from({ length: height }, (_, index) => {
      const removed = item.removed[index]
      const added = item.added[index]
      return {
        left: removed ? cell(removed, language, pairs[index]?.removed ?? null, removed.oldNumber) : EMPTY_CELL,
        right: added ? cell(added, language, pairs[index]?.added ?? null, added.newNumber) : EMPTY_CELL
      }
    })
  })
}

function groupItems(lines: readonly DiffLine[]): readonly Item[] {
  const items: Item[] = []
  let removed: DiffLine[] = []
  let added: DiffLine[] = []
  const flush = (): void => {
    if (removed.length === 0 && added.length === 0) return
    items.push({ kind: 'run', removed, added })
    removed = []
    added = []
  }
  for (const line of lines) {
    if (line.kind === 'CONTEXT') {
      flush()
      items.push({ kind: 'context', line })
    } else if (line.kind === 'REMOVED') {
      removed.push(line)
    } else {
      added.push(line)
    }
  }
  flush()
  return items
}

function pairedSegments(
  removed: readonly DiffLine[],
  added: readonly DiffLine[]
): readonly ({ removed: readonly TextSegment[]; added: readonly TextSegment[] } | null)[] {
  return removed.map((line, index) => {
    const partner = added[index]
    return partner ? wordDiff(line.text, partner.text) : null
  })
}

function unifiedRow(line: DiffLine, language: CodeLanguage, segments: readonly TextSegment[] | null): UnifiedRow {
  return { kind: line.kind, oldNumber: line.oldNumber, newNumber: line.newNumber, parts: codeParts(line.text, language, segments) }
}

function cell(line: DiffLine, language: CodeLanguage, segments: readonly TextSegment[] | null, number: number | null): SplitCell {
  return { kind: line.kind, number, parts: codeParts(line.text, language, segments) }
}
