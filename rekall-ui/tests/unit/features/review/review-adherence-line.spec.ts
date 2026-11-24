import { describe, expect, it } from 'vitest'
import { adherenceLine } from '@/features/review/lib/review-adherence-line'
import { adherence } from '../adherence/fixtures'

describe('adherenceLine', () => {
  it('puts the three figures on one line', () => {
    expect(adherenceLine(adherence())).toBe('2 of 3 accepted first time · 1 sent back · 1 deviation')
  })

  it('pluralises deviations', () => {
    expect(adherenceLine(adherence({ deviations: 0 }))).toContain('0 deviations')
  })

  it('says no step was accepted yet instead of counting against zero', () => {
    expect(adherenceLine(adherence({ stepsDone: 0, acceptedFirstTime: 0, sentBack: 0, deviations: 0 }))).toBe('No step accepted yet · 0 sent back · 0 deviations')
  })

  it('says there is no plan to measure for a task without steps', () => {
    expect(adherenceLine(adherence({ stepsTotal: 0, stepsDone: 0 }))).toBe('This task has no steps, so there is no plan to measure.')
  })
})
