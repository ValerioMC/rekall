export type TubeShares = Readonly<{ done: number; claimed: number; running: number }>

/**
 * How much of a plan tube each state fills, as percentages of its length. A running step is one
 * step's share, and nothing is drawn past the last step.
 */
export function tubeShares(done: number, claimed: number, total: number, live: boolean): TubeShares {
  if (total <= 0) return { done: 0, claimed: 0, running: 0 }
  const share = (steps: number): number => (steps / total) * 100
  const settled = Math.min(done + claimed, total)
  const doneShare = share(Math.min(done, total))
  const claimedShare = share(settled) - doneShare
  const runningShare = live ? Math.min(share(1), share(total - settled)) : 0
  return { done: doneShare, claimed: claimedShare, running: runningShare }
}

export function isPlanComplete(done: number, total: number): boolean {
  return total > 0 && done >= total
}
