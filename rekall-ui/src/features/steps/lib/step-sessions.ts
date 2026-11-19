import type { TaskStep } from '@/features/steps/model/task-step'

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

export type SessionStanding = 'accepted' | 'working' | 'waiting' | 'idle'

export function sessionStanding(steps: readonly TaskStep[]): SessionStanding {
  if (steps.every((step) => step.state === 'DONE')) return 'accepted'
  if (steps.some((step) => step.state === 'RUNNING')) return 'working'
  if (steps.some((step) => step.state === 'CLAIMED')) return 'waiting'
  return 'idle'
}

export function sessionNote(steps: readonly TaskStep[]): string {
  const claimed = steps.filter((step) => step.state === 'CLAIMED').length
  const count = `${steps.length} ${steps.length === 1 ? 'step' : 'steps'}`
  const standing = sessionStanding(steps)
  if (standing === 'accepted') return `Accepted · ${count}`
  if (standing === 'working') return `Working now · ${claimed} of ${steps.length} claimed`
  if (standing === 'waiting') return `Waiting for you · ${claimed} claimed`
  return `Not started · ${count}`
}

/** The sessions as the list shows them: with `hideDone` the accepted steps drop out and a session left empty goes with them. */
export function visibleSessions(sessions: readonly StepSession[], hideDone: boolean): StepSession[] {
  if (!hideDone) return [...sessions]
  return sessions
    .map((session) => ({ ...session, steps: session.steps.filter((step) => !step.done) }))
    .filter((session) => session.steps.length > 0)
}
