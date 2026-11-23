import type { Decision, DecisionEdit, NewDecision } from '@/features/decisions/model/decision'

export interface EditDraft {
  readonly answer: string
  readonly note: string
}

export interface AddDraft {
  readonly topic: string
  readonly question: string
  readonly answer: string
  /** The step's number on the checklist as text, or empty for the whole task. */
  readonly step: string
}

export const EMPTY_ADD_DRAFT: AddDraft = { topic: '', question: '', answer: '', step: '' }

/** What the edit form starts with: the answer as it stands, or nothing for a question not yet answered. */
export function startEditDraft(decision: Decision): EditDraft {
  return { answer: decision.state === 'ANSWERED' ? (decision.answer ?? '') : '', note: '' }
}

function blankToNull(text: string): string | null {
  const trimmed = text.trim()
  return trimmed === '' ? null : trimmed
}

/** The reason an edit cannot be saved, said where the answer is, or null when it can. */
export function editProblem(decision: Decision, draft: EditDraft): string | null {
  const answer = blankToNull(draft.answer)
  if (decision.state !== 'ANSWERED') return answer === null ? 'Give it an answer. It stays open until then.' : null
  const changed = answer !== null && answer !== (decision.answer ?? '').trim()
  if (!changed && blankToNull(draft.note) === null) return 'Change the answer or add a note. Nothing else is saved here.'
  return null
}

export function toEdit(decision: Decision, draft: EditDraft): DecisionEdit {
  const answer = blankToNull(draft.answer)
  const unchanged = decision.state === 'ANSWERED' && answer === (decision.answer ?? '').trim()
  return { answer: unchanged ? null : answer, note: blankToNull(draft.note) }
}

export interface AddProblem {
  readonly field: 'question' | 'answer' | 'form'
  readonly message: string
}

/** The first thing missing from a new decision, with the field it belongs to. */
export function addProblem(draft: AddDraft): AddProblem | null {
  if (blankToNull(draft.question) === null) return { field: 'question', message: 'Say what was decided about. A decision needs its question.' }
  if (blankToNull(draft.answer) === null) return { field: 'answer', message: 'Say what was decided. A decision needs its answer.' }
  return null
}

export function toNewDecision(draft: AddDraft): NewDecision {
  const step = Number.parseInt(draft.step, 10)
  return {
    topic: blankToNull(draft.topic),
    question: draft.question.trim(),
    answer: draft.answer.trim(),
    stepNumber: Number.isNaN(step) ? null : step
  }
}

/** The text of a failed write, as the server worded it. */
export function failureText(error: unknown): string {
  return error instanceof Error ? error.message : 'The decision could not be saved.'
}
