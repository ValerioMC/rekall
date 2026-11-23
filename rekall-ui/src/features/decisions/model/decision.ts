import type { DecisionId, TaskId } from '@/model/branded'

export type DecisionState = 'OPEN' | 'ANSWERED' | 'ON_HOLD'

/** Where a decision was taken: a terminal session, a chat session, or the person editing the record by hand. */
export type DecisionSource = 'TERMINAL' | 'CHAT' | 'PERSON'

export type Speaker = 'CLAUDE' | 'PERSON'

export interface DecisionOption {
  readonly label: string
  readonly effect: string | null
  readonly recommended: boolean
}

export interface DecisionLine {
  readonly speaker: Speaker
  readonly text: string
  readonly spokenAt: string
}

export interface DecisionDetail {
  readonly text: string
  readonly addedAt: string
}

/**
 * One question about a task and what became of it. `stepNumber` is the step's place on the
 * checklist, counted from 1, and empty when the decision is about the task as a whole.
 * `answer` is set only once answered and `holdNote` only while on hold.
 */
export interface Decision {
  readonly id: DecisionId
  readonly taskId: TaskId
  readonly key: string
  readonly topic: string | null
  readonly question: string
  readonly why: string | null
  readonly stepNumber: number | null
  readonly stepTitle: string | null
  readonly state: DecisionState
  readonly answer: string | null
  readonly holdNote: string | null
  readonly previousAnswer: string | null
  readonly changedAt: string | null
  readonly source: DecisionSource
  readonly options: readonly DecisionOption[]
  readonly lines: readonly DecisionLine[]
  readonly details: readonly DecisionDetail[]
}

/** A task's decisions and the text its context carries for them, as the server renders it. */
export interface TaskDecisions {
  readonly decisions: readonly Decision[]
  readonly sessionBlock: string
}

export interface NewDecision {
  readonly topic: string | null
  readonly question: string
  readonly answer: string
  readonly stepNumber: number | null
}

export interface DecisionEdit {
  readonly answer: string | null
  readonly note: string | null
}

export const NO_DECISIONS: TaskDecisions = { decisions: [], sessionBlock: '' }

export function isOpen(decision: Decision): boolean {
  return decision.state === 'OPEN'
}

export function isAnswered(decision: Decision): boolean {
  return decision.state === 'ANSWERED'
}

export function isOnHold(decision: Decision): boolean {
  return decision.state === 'ON_HOLD'
}
