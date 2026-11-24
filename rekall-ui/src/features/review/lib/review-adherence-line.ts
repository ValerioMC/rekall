import type { PlanAdherence } from '@/features/adherence/model/adherence'

/** A task's adherence in one line under an open claim: the same three figures the task page shows. */
export function adherenceLine(adherence: PlanAdherence): string {
  if (!adherence.stepsTotal) return 'This task has no steps, so there is no plan to measure.'
  const deviations = `${adherence.deviations} ${adherence.deviations === 1 ? 'deviation' : 'deviations'}`
  const accepted = adherence.stepsDone ? `${adherence.acceptedFirstTime} of ${adherence.stepsDone} accepted first time` : 'No step accepted yet'
  return `${accepted} · ${adherence.sentBack} sent back · ${deviations}`
}
