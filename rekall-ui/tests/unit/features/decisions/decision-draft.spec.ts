import { describe, expect, it } from 'vitest'
import { addProblem, EMPTY_ADD_DRAFT, editProblem, startEditDraft, toEdit, toNewDecision } from '@/features/decisions/lib/decision-draft'
import { answered, decision } from './fixtures'

describe('the edit draft', () => {
  it('starts from the answer as it stands, and empty for a question not yet answered', () => {
    expect(startEditDraft(answered('replicas', '3'))).toEqual({ answer: '3', note: '' })
    expect(startEditDraft(decision('region'))).toEqual({ answer: '', note: '' })
  })

  it('asks an unanswered decision for its answer', () => {
    expect(editProblem(decision('region'), { answer: ' ', note: 'only a note' })).toContain('Give it an answer')
    expect(editProblem(decision('region'), { answer: 'eu-west-1', note: '' })).toBeNull()
  })

  it('asks an answered decision for a changed answer or a note', () => {
    const replicas = answered('replicas', '3')

    expect(editProblem(replicas, { answer: '3', note: '' })).toContain('Change the answer or add a note')
    expect(editProblem(replicas, { answer: '4', note: '' })).toBeNull()
    expect(editProblem(replicas, { answer: '3', note: 'Per zone.' })).toBeNull()
  })

  it('sends only what changed: an unchanged answer travels as none, blank text as none', () => {
    const replicas = answered('replicas', '3')

    expect(toEdit(replicas, { answer: '3', note: ' Per zone. ' })).toEqual({ answer: null, note: 'Per zone.' })
    expect(toEdit(replicas, { answer: '4', note: '' })).toEqual({ answer: '4', note: null })
    expect(toEdit(decision('region'), { answer: 'eu-west-1', note: '' })).toEqual({ answer: 'eu-west-1', note: null })
  })
})

describe('the add draft', () => {
  it('needs a question and an answer', () => {
    expect(addProblem(EMPTY_ADD_DRAFT)).toMatchObject({ field: 'question', message: expect.stringContaining('needs its question') })
    expect(addProblem({ ...EMPTY_ADD_DRAFT, question: 'Which namespace?' })).toMatchObject({ field: 'answer', message: expect.stringContaining('needs its answer') })
    expect(addProblem({ ...EMPTY_ADD_DRAFT, question: 'Which namespace?', answer: 'billing' })).toBeNull()
  })

  it('becomes a decision with trimmed text, no topic when blank, and the step as a number', () => {
    expect(toNewDecision({ topic: ' ', question: ' Which namespace? ', answer: ' billing ', step: '' })).toEqual({ topic: null, question: 'Which namespace?', answer: 'billing', stepNumber: null })
    expect(toNewDecision({ topic: 'Cluster', question: 'Q?', answer: 'a', step: '2' }).stepNumber).toBe(2)
  })
})
