import type { TaskStep } from '@/features/steps/model/task-step'

export type StepEventKind = 'created' | 'sent-back' | 'running' | 'claimed' | 'done'

export interface StepEvent {
  readonly kind: StepEventKind
  readonly label: string
  readonly at: string
}

/** What happened to a step, oldest first: created, each send-back, the session at work, the claim, the acceptance. */
export function historyOf(step: TaskStep): StepEvent[] {
  const events: StepEvent[] = [{ kind: 'created', label: 'Created', at: step.createdAt }]
  step.passes.forEach((pass, index) =>
    events.push({ kind: 'sent-back', label: `Pass ${index + 1} sent back`, at: pass.sentBackAt })
  )
  if (step.runningAt) events.push({ kind: 'running', label: 'A session started', at: step.runningAt })
  if (step.claimedAt) events.push({ kind: 'claimed', label: 'Claimed', at: step.claimedAt })
  if (step.doneAt) events.push({ kind: 'done', label: 'Accepted', at: step.doneAt })
  return events.sort((first, second) => Date.parse(first.at) - Date.parse(second.at))
}
