import type { TaskStep } from '@/features/steps/model/task-step'

export const ADHERENCE_HELP = 'A high share of steps sent back usually means steps are too large or their detail too vague.'

/** One pass at a step: one that was sent back, the accepted one, or the one a session has in hand. */
export type PassKind = 'sent-back' | 'accepted' | 'in-hand'

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
  if (!hasStarted(step)) return sentBack
  return [...sentBack, step.state === 'DONE' ? 'accepted' : 'in-hand']
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

export interface FlowVisit {
  readonly visits: number
  readonly budgetSpent: boolean
}

/** How many steps the flow entered and how many entries that was, for the line that folds the list. */
export function visitsSummary(visits: readonly FlowVisit[]): string {
  const entries = visits.reduce((sum, entry) => sum + entry.visits, 0)
  return `${visits.length} ${visits.length === 1 ? 'step' : 'steps'}, ${entries} ${entries === 1 ? 'visit' : 'visits'}`
}

export function spentBudgets(visits: readonly FlowVisit[]): number {
  return visits.filter((entry) => entry.budgetSpent).length
}
