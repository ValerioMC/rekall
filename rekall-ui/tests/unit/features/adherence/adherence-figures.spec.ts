import { describe, expect, it } from 'vitest'
import { passesOf, passesSummary, shareDetail, stepsLabel } from '@/features/adherence/lib/adherence-figures'
import { step } from './fixtures'

describe('the passes of a step', () => {
  it('are one segment for each send-back and one for the pass under way', () => {
    expect(passesOf(step(1, 'DONE', 2))).toEqual(['sent-back', 'sent-back', 'current'])
    expect(passesOf(step(1, 'RUNNING'))).toEqual(['current'])
  })

  it('are only the earlier ones for a step sent back and not started again', () => {
    expect(passesOf(step(1, 'OPEN', 1))).toEqual(['sent-back'])
  })

  it('are none for a step nobody started', () => {
    expect(passesOf(step(1, 'OPEN'))).toEqual([])
    expect(passesSummary(step(1, 'OPEN'))).toBe('not started')
  })

  it('are described in words, for those who cannot tell the segments apart', () => {
    expect(passesSummary(step(1, 'DONE'))).toBe('1 pass')
    expect(passesSummary(step(1, 'DONE', 2))).toBe('3 passes, 2 sent back')
  })
})

describe('the share of steps accepted first time', () => {
  it('reads as a whole percentage', () => {
    expect(shareDetail(2 / 3)).toBe('67% of the accepted steps')
  })

  it('says so when no step is accepted, instead of showing zero', () => {
    expect(shareDetail(null)).toBe('no step accepted yet')
  })
})

describe('naming steps', () => {
  it('uses the singular for one', () => {
    expect(stepsLabel([2])).toBe('step 2')
    expect(stepsLabel([2, 5])).toBe('steps 2, 5')
  })
})
