export type ChangeBlock = 'added' | 'removed' | 'none'

const BLOCK_COUNT = 5

/** Five blocks that split a file's changed lines between added and removed; all `none` when nothing changed. */
export function changeBlocks(additions: number, deletions: number): readonly ChangeBlock[] {
  const total = additions + deletions
  if (total === 0) return Array.from({ length: BLOCK_COUNT }, () => 'none')
  const added = Math.min(BLOCK_COUNT, Math.max(additions > 0 ? 1 : 0, Math.round((additions / total) * BLOCK_COUNT)))
  return Array.from({ length: BLOCK_COUNT }, (_, index) => (index < added ? 'added' : 'removed'))
}
