import type { TaskStep } from '@/features/steps/model/task-step'

export const ADHERENCE_HELP = 'A high share of steps sent back usually means steps are too large or their detail too vague.'

/** One pass at a step: one that was sent back, or the one now under way or accepted. */
export type PassKind = 'sent-back' | 'current'

/** The share of accepted steps that needed one pass, as a whole percentage; none before a step is accepted. */
export function shareDetail(share: number | null): string {
  return share === null ? 'no step accepted yet' : `${Math.round(share * 100)}% of the accepted steps`
}

function hasStarted(step: TaskStep): boolean {
  return step.state === 'RUNNING' || step.state === 'CLAIMED' || step.state === 'DONE'
}

/** The passes of a step in order: each send-back, then the pass the step is in once started. */
export function passesOf(step: TaskStep): PassKind[] {
  const sentBack = step.passes.map((): PassKind => 'sent-back')
  return hasStarted(step) ? [...sentBack, 'current'] : sentBack
}

export function passesSummary(step: TaskStep): string {
  const passes = passesOf(step)
  if (!passes.length) return 'not started'
  const sentBack = step.passes.length
  const total = `${passes.length} ${passes.length === 1 ? 'pass' : 'passes'}`
  return sentBack ? `${total}, ${sentBack} sent back` : total
}

export function stepsLabel(numbers: readonly number[]): string {
  return numbers.length === 1 ? `step ${numbers[0]}` : `steps ${numbers.join(', ')}`
}
