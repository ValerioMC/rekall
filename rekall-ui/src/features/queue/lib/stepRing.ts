export const MAX_RING_SLICES = 12
const SLICE_GAP = 1.8

export interface RingProgress {
  readonly total: number
  readonly settled: number
}

export type RingSliceState = 'settled' | 'current' | 'pending'

export interface RingSlice {
  readonly state: RingSliceState
  readonly dashArray: string
  readonly dashOffset: number
}

export type StepRing =
  | { readonly kind: 'slices'; readonly slices: readonly RingSlice[] }
  | { readonly kind: 'arc'; readonly dashArray: string }

function lengthOf(value: number): string {
  return value.toFixed(2)
}

function sliceStateAt(index: number, settled: number): RingSliceState {
  if (index < settled) return 'settled'
  return index === settled ? 'current' : 'pending'
}

function sliceAt(index: number, progress: RingProgress, circumference: number): RingSlice {
  const share = circumference / progress.total
  const gap = progress.total === 1 ? 0 : SLICE_GAP
  const length = share - gap
  return {
    state: sliceStateAt(index, progress.settled),
    dashArray: `${lengthOf(length)} ${lengthOf(circumference - length)}`,
    dashOffset: -(index * share + gap / 2)
  }
}

/**
 * The ring a running task's steps draw on a circle of `circumference`, starting at its top: one
 * slice per step up to `MAX_RING_SLICES`, beyond that one arc filled in proportion to the settled.
 */
export function stepRing(progress: RingProgress, circumference: number): StepRing {
  const settled = Math.min(Math.max(progress.settled, 0), progress.total)
  const clamped: RingProgress = { total: progress.total, settled }
  if (progress.total > MAX_RING_SLICES) {
    const filled = (circumference * settled) / progress.total
    return { kind: 'arc', dashArray: `${lengthOf(filled)} ${lengthOf(circumference)}` }
  }
  const slices = Array.from({ length: progress.total }, (_, index) => sliceAt(index, clamped, circumference))
  return { kind: 'slices', slices }
}
