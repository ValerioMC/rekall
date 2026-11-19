import { describe, expect, it } from 'vitest'
import { isPlanComplete, tubeShares } from '@/common/plan/tube-shares'

describe('tubeShares', () => {
  it('draws nothing for a plan without steps', () => {
    expect(tubeShares(0, 0, 0, true)).toEqual({ done: 0, claimed: 0, running: 0 })
  })

  it('splits the tube into done, claimed and the step a session is on', () => {
    expect(tubeShares(2, 1, 5, true)).toEqual({ done: 40, claimed: 20, running: 20 })
  })

  it('leaves the running share out when no session works', () => {
    expect(tubeShares(2, 1, 5, false).running).toBe(0)
  })

  it('never draws past the last step', () => {
    expect(tubeShares(4, 1, 5, true)).toEqual({ done: 80, claimed: 20, running: 0 })
    expect(tubeShares(7, 0, 5, false).done).toBe(100)
  })
})

describe('isPlanComplete', () => {
  it('is true only when every step is done', () => {
    expect(isPlanComplete(3, 3)).toBe(true)
    expect(isPlanComplete(2, 3)).toBe(false)
    expect(isPlanComplete(0, 0)).toBe(false)
  })
})
