import { formatDateTime, formatTimeOfDay } from '@/common/format/app-locale'
import type { Decision, DecisionSource, DecisionState } from '@/features/decisions/model/decision'

export const STATE_LABEL: Readonly<Record<DecisionState, string>> = {
  OPEN: 'Open',
  ANSWERED: 'Answered',
  ON_HOLD: 'On hold'
}

export const SOURCE_LABEL: Readonly<Record<DecisionSource, string>> = {
  TERMINAL: 'Terminal',
  CHAT: 'Chat',
  PERSON: 'Edited by you'
}

/** Where an open question is being discussed, in the words of the row. */
export function discussedIn(source: DecisionSource): string {
  if (source === 'TERMINAL') return 'the terminal'
  return source === 'CHAT' ? 'the chat' : 'the conversation'
}

/** Who asked, for the line under the question. */
export function askedBy(decision: Decision): string {
  return decision.source === 'PERSON' ? 'You wrote this' : 'Asked in the session'
}

/** The scope tag a row carries when the task has steps: the step it is read at, or the whole task. */
export function scopeLabel(decision: Decision): string {
  return decision.stepNumber === null ? 'Task' : `Step ${decision.stepNumber}`
}

/** The time of day when it was today, the date with it otherwise. */
export function timeOf(iso: string, now: Date = new Date()): string {
  const at = new Date(iso)
  return at.toDateString() === now.toDateString() ? formatTimeOfDay(at) : formatDateTime(at)
}

/** Where a session reads the decision, in a sentence. */
export function readWhen(decision: Decision): string {
  if (decision.stepNumber === null) return 'Read by every session that runs this task, whichever step it works on.'
  const title = decision.stepTitle === null ? '' : `, ${decision.stepTitle},`
  return `Read when step ${decision.stepNumber}${title} runs, and by any session that runs the whole task.`
}
