import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { usePlanStore } from '@/stores/plan.store'

vi.mock('@/api/plan.api', () => ({
  NO_ANSWERS: { backfillDefaults: {}, confirmedDrops: [] },
  fetchPlan: vi.fn(async () => ({
    planId: 'p',
    applicable: false,
    blockedCount: 0,
    awaitingInputCount: 1,
    statements: []
  })),
  planWithAnswers: vi.fn(async () => ({
    planId: 'p',
    applicable: true,
    blockedCount: 0,
    awaitingInputCount: 0,
    statements: []
  })),
  applyPlan: vi.fn(async () => ({ planId: 'p', statementCount: 3, documentsDeleted: 0 }))
}))

describe('plan store', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('collects only the answers that were actually given', () => {
    const plan = usePlanStore()

    plan.setBackfill('project.code', '  ')
    plan.setBackfill('project.name', 'UNKNOWN')
    plan.setConfirmation('project.scratch', true)
    plan.setConfirmation('project.keep', false)

    expect(plan.answers.backfillDefaults).toEqual({ 'project.name': 'UNKNOWN' })
    expect(plan.answers.confirmedDrops).toEqual(['project.scratch'])
  })

  it('re-plans with the answers rather than patching the plan on screen', async () => {
    const plan = usePlanStore()
    const api = await import('@/api/plan.api')

    await plan.refresh()
    expect(api.fetchPlan).toHaveBeenCalledOnce()

    plan.setBackfill('project.code', 'UNKNOWN')
    await plan.refresh()

    expect(api.planWithAnswers).toHaveBeenCalledWith({
      backfillDefaults: { 'project.code': 'UNKNOWN' },
      confirmedDrops: []
    })
    expect(plan.plan?.applicable).toBe(true)
  })

  it('clears the answers once they have been applied', async () => {
    const plan = usePlanStore()
    plan.setBackfill('project.code', 'UNKNOWN')
    plan.setConfirmation('old_table', true)

    const result = await plan.apply()

    expect(result.statementCount).toBe(3)
    expect(plan.answers.backfillDefaults).toEqual({})
    expect(plan.answers.confirmedDrops).toEqual([])
  })
})
