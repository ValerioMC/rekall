import { apiClient, request } from './client'
import { ApplyResultSchema, PlanSchema } from './schemas/plan.schema'
import type { ApplyResult, Plan, PlanAnswers } from '@/model/plan'

export const NO_ANSWERS: PlanAnswers = { backfillDefaults: {}, confirmedDrops: [] }

export async function fetchPlan(): Promise<Plan> {
  return request(async () => PlanSchema.parse(await apiClient('/api/meta/plan')))
}

/**
 * Re-plans with the user's answers.
 *
 * The preview and the execution must be the same plan, so answering a question re-computes it
 * rather than patching the one already on screen.
 */
export async function planWithAnswers(answers: PlanAnswers): Promise<Plan> {
  return request(async () =>
    PlanSchema.parse(await apiClient('/api/meta/plan', { method: 'POST', body: answers }))
  )
}

export async function applyPlan(answers: PlanAnswers): Promise<ApplyResult> {
  return request(async () =>
    ApplyResultSchema.parse(await apiClient('/api/meta/apply', { method: 'POST', body: answers }))
  )
}
