import type { TaskStep } from '@/features/catalog/model/catalog'

/** The steps that run in one Claude session, in checklist order. */
export interface StepSession {
  readonly number: number
  readonly steps: readonly TaskStep[]
}

/** Whether the step at `index` opens a session of its own: the first one always does, silently. */
export function opensSession(steps: readonly TaskStep[], index: number): boolean {
  return index > 0 && steps[index]?.startsSession === true
}

/** Cuts the checklist at every step that opens a session; a list with none is one session. */
export function groupBySession(steps: readonly TaskStep[]): StepSession[] {
  const sessions: TaskStep[][] = []
  steps.forEach((step, index) => {
    if (index === 0 || opensSession(steps, index)) sessions.push([])
    sessions[sessions.length - 1]!.push(step)
  })
  return sessions.map((grouped, index) => ({ number: index + 1, steps: grouped }))
}
