import { describe, expect, it } from 'vitest'
import { planSummary } from '@/features/steps/lib/plan-summary'

describe('planSummary', () => {
  it('says who has to act when steps are claimed, ahead of anything running', () => {
    expect(planSummary({ done: 2, claimed: 1, total: 5, runningNumber: 4 })).toEqual({
      lead: '1 step is claimed and waits for you.',
      tally: '2 of 5 done',
      waitsForReader: true
    })
  })

  it('pluralises the claimed steps', () => {
    expect(planSummary({ done: 0, claimed: 3, total: 3, runningNumber: null }).lead).toBe(
      '3 steps are claimed and wait for you.'
    )
  })

  it('names the running step when nothing waits', () => {
    expect(planSummary({ done: 1, claimed: 0, total: 4, runningNumber: 2 })).toEqual({
      lead: 'A session is working on step 2.',
      tally: '1 of 4 done',
      waitsForReader: false
    })
  })

  it('is only a tally when nothing waits and nothing runs', () => {
    expect(planSummary({ done: 4, claimed: 0, total: 4, runningNumber: null })).toEqual({
      lead: '',
      tally: '4 of 4 done',
      waitsForReader: false
    })
  })
})
