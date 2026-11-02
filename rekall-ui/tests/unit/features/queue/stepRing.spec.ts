import { describe, expect, it } from 'vitest'
import { MAX_RING_SLICES, stepRing } from '@/features/queue/lib/stepRing'

const CIRCUMFERENCE = 100

describe('the step ring', () => {
  it('draws one slice per step, the settled filled, the next one current, the rest pending', () => {
    const ring = stepRing({ total: 5, settled: 2 }, CIRCUMFERENCE)

    expect(ring.kind).toBe('slices')
    if (ring.kind !== 'slices') return
    expect(ring.slices.map((slice) => slice.state)).toEqual(['settled', 'settled', 'current', 'pending', 'pending'])
  })

  it('spaces the slices evenly round the circle with a gap between them', () => {
    const ring = stepRing({ total: 4, settled: 0 }, CIRCUMFERENCE)

    if (ring.kind !== 'slices') throw new Error('expected slices')
    expect(ring.slices.map((slice) => slice.dashArray)).toEqual(Array(4).fill('23.20 76.80'))
    expect(ring.slices.map((slice) => slice.dashOffset)).toEqual([-0.9, -25.9, -50.9, -75.9])
  })

  it('closes a single step into a whole ring with no gap', () => {
    const ring = stepRing({ total: 1, settled: 0 }, CIRCUMFERENCE)

    if (ring.kind !== 'slices') throw new Error('expected slices')
    expect(ring.slices).toEqual([{ state: 'current', dashArray: '100.00 0.00', dashOffset: -0 }])
  })

  it('has no current slice once every step is settled', () => {
    const ring = stepRing({ total: 3, settled: 3 }, CIRCUMFERENCE)

    if (ring.kind !== 'slices') throw new Error('expected slices')
    expect(ring.slices.every((slice) => slice.state === 'settled')).toBe(true)
  })

  it('keeps slices up to the limit and switches to a proportional arc above it', () => {
    expect(stepRing({ total: MAX_RING_SLICES, settled: 1 }, CIRCUMFERENCE).kind).toBe('slices')

    const ring = stepRing({ total: MAX_RING_SLICES * 2, settled: MAX_RING_SLICES }, CIRCUMFERENCE)

    expect(ring).toEqual({ kind: 'arc', dashArray: '50.00 100.00' })
  })

  it('never fills past the whole ring', () => {
    const ring = stepRing({ total: 20, settled: 25 }, CIRCUMFERENCE)

    expect(ring).toEqual({ kind: 'arc', dashArray: '100.00 100.00' })
  })
})
