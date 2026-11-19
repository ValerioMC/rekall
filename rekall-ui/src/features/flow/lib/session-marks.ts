import { routeSessionRelation, type StepFlow } from '@/features/flow/lib/step-flow'
import type { StepLink } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/** How much taller than a plain step the node of a step that opens a session is. */
export const SESSION_RIBBON_HEIGHT = 22

/**
 * Accents a session cycles through, kept off the amber of the current step and the cyan of the
 * selection. The number on the chip identifies a session when colour does not.
 */
const SESSION_ACCENTS: readonly string[] = [
  'var(--color-epic)',
  'var(--color-kind-concept)',
  'var(--color-kind-decision)',
  'var(--color-kind-action)'
]

/** What a step's node says about the Claude session it runs in. */
export interface StepSessionMark {
  readonly number: number
  readonly opens: boolean
  readonly accent: string
  readonly description: string
}

/** What a route says about the sessions: a gate where it enters one, a tint where it stays in one. */
export type RouteSessionMark =
  | {
      readonly kind: 'gate'
      readonly opens: boolean
      readonly label: string
      readonly description: string
      readonly accent: string
    }
  | { readonly kind: 'shared'; readonly accent: string }

export function sessionAccent(number: number): string {
  return SESSION_ACCENTS[(number - 1) % SESSION_ACCENTS.length]!
}

/** The mark of `stepId`, or null when the task runs in one session and nothing is marked. */
export function sessionMarkOf(flow: StepFlow, stepId: TaskStepId): StepSessionMark | null {
  const number = flow.sessionOf.get(stepId)
  if (flow.sessionCount < 2 || number === undefined) return null
  const opens = flow.sessionOpeners.has(stepId)
  return { number, opens, accent: sessionAccent(number), description: describeSession(flow, stepId, number, opens) }
}

/**
 * A route into a step that opens a session always gets a gate, even from a step of that same
 * session, since entering it starts one; a route into another session's step gets one too.
 */
export function routeSessionMark(link: StepLink, flow: StepFlow): RouteSessionMark | null {
  const relation = routeSessionRelation(link, flow)
  if (relation === 'none' || link.toStepId === null) return null
  const number = flow.sessionOf.get(link.toStepId)!
  const accent = sessionAccent(number)
  if (flow.sessionOpeners.has(link.toStepId)) {
    return {
      kind: 'gate',
      opens: true,
      label: `S${number}`,
      description: `Enters session ${number}, a new Claude session`,
      accent
    }
  }
  if (relation === 'crosses') {
    return { kind: 'gate', opens: false, label: `S${number}`, description: `Moves into session ${number}`, accent }
  }
  return { kind: 'shared', accent }
}

export function sessionRibbonHeights(flow: StepFlow): Map<TaskStepId, number> {
  return new Map([...flow.sessionOpeners].map((id) => [id, SESSION_RIBBON_HEIGHT]))
}

function describeSession(flow: StepFlow, stepId: TaskStepId, number: number, opens: boolean): string {
  if (opens) return `session ${number}, opens a new session`
  if (flow.steps[0]?.id === stepId) return `session ${number}, where the checklist begins`
  return `session ${number}, shares the session of the step before`
}
