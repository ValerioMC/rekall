export type DiffHunkKind = 'same' | 'added' | 'removed'

/** A run of consecutive lines that were kept, added or removed. */
export interface DiffHunk {
  readonly kind: DiffHunkKind
  readonly lines: readonly string[]
}

/** The line-by-line difference between two texts, as runs in reading order. */
export function diffLines(before: string, after: string): DiffHunk[] {
  const left = before.split('\n')
  const right = after.split('\n')
  const lengths = commonLengths(left, right)
  const hunks: DiffHunk[] = []
  let l = 0
  let r = 0
  while (l < left.length || r < right.length) {
    if (l < left.length && r < right.length && lineAt(left, l) === lineAt(right, r)) {
      append(hunks, 'same', lineAt(left, l))
      l += 1
      r += 1
    } else if (l < left.length && (r === right.length || commonAt(lengths, l + 1, r) >= commonAt(lengths, l, r + 1))) {
      append(hunks, 'removed', lineAt(left, l))
      l += 1
    } else {
      append(hunks, 'added', lineAt(right, r))
      r += 1
    }
  }
  return hunks
}

/** `lengths[l][r]`: how many lines the tails `left[l..]` and `right[r..]` have in common, in order. */
function commonLengths(left: readonly string[], right: readonly string[]): number[][] {
  const lengths = Array.from({ length: left.length + 1 }, () => new Array<number>(right.length + 1).fill(0))
  for (let l = left.length - 1; l >= 0; l -= 1) {
    for (let r = right.length - 1; r >= 0; r -= 1) {
      lengths[l]![r] = lineAt(left, l) === lineAt(right, r) ? commonAt(lengths, l + 1, r + 1) + 1 : Math.max(commonAt(lengths, l + 1, r), commonAt(lengths, l, r + 1))
    }
  }
  return lengths
}

function lineAt(lines: readonly string[], index: number): string {
  return lines[index] ?? ''
}

function commonAt(lengths: readonly (readonly number[])[], l: number, r: number): number {
  return lengths[l]?.[r] ?? 0
}

function append(hunks: DiffHunk[], kind: DiffHunkKind, line: string): void {
  const last = hunks[hunks.length - 1]
  if (last !== undefined && last.kind === kind) {
    hunks[hunks.length - 1] = { kind, lines: [...last.lines, line] }
  } else {
    hunks.push({ kind, lines: [line] })
  }
}

/** A run of kept lines folded away, which the reader can open. */
export interface FoldedRun {
  readonly kind: 'folded'
  readonly lines: readonly string[]
}

export type DiffRow = DiffHunk | FoldedRun

const FOLD_CONTEXT = 1
const FOLD_FROM = 4

/** Keeps `FOLD_CONTEXT` lines either side of a change and folds the middle of a long kept run. */
export function foldUnchanged(hunks: readonly DiffHunk[]): DiffRow[] {
  return hunks.flatMap((hunk, index): DiffRow[] => {
    if (hunk.kind !== 'same' || hunk.lines.length < FOLD_FROM) return [hunk]
    const head = index === 0 ? 0 : FOLD_CONTEXT
    const tail = index === hunks.length - 1 ? 0 : FOLD_CONTEXT
    const hidden = hunk.lines.slice(head, hunk.lines.length - tail)
    if (hidden.length < 2) return [hunk]
    const rows: DiffRow[] = []
    if (head > 0) rows.push({ kind: 'same', lines: hunk.lines.slice(0, head) })
    rows.push({ kind: 'folded', lines: hidden })
    if (tail > 0) rows.push({ kind: 'same', lines: hunk.lines.slice(hunk.lines.length - tail) })
    return rows
  })
}
