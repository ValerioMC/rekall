import { describe, expect, it } from 'vitest'
import { DecisionStreamEventSchema, TaskDecisionsResponseSchema } from '@/features/decisions/api/schemas/decision.schema'

const response = {
  id: '11111111-1111-1111-1111-111111111111',
  key: 'db-password',
  topic: 'Secrets',
  question: 'Where does the password live?',
  why: null,
  stepNumber: 3,
  stepTitle: 'Wire the ingress',
  state: 'OPEN',
  answer: null,
  holdNote: null,
  previousAnswer: null,
  changedAt: null,
  source: 'TERMINAL',
  options: [{ label: 'Vault', effect: 'Rotation', recommended: true }],
  lines: [{ speaker: 'CLAUDE', text: 'Which store?', spokenAt: '2026-11-23T12:00:00Z' }],
  details: [{ text: 'Prod only.', addedAt: '2026-11-23T12:05:00Z' }]
}

describe('the decision schemas', () => {
  it('reads the routes answer with the text a session reads', () => {
    const parsed = TaskDecisionsResponseSchema.parse({ decisions: [response], sessionBlock: '<decisions/>' })

    expect(parsed.decisions[0]).toMatchObject({ key: 'db-password', stepNumber: 3, state: 'OPEN', source: 'TERMINAL' })
    expect(parsed.sessionBlock).toBe('<decisions/>')
  })

  it('refuses a state or a source the server does not use', () => {
    expect(TaskDecisionsResponseSchema.safeParse({ decisions: [{ ...response, state: 'MAYBE' }], sessionBlock: '' }).success).toBe(false)
    expect(TaskDecisionsResponseSchema.safeParse({ decisions: [{ ...response, source: 'EMAIL' }], sessionBlock: '' }).success).toBe(false)
  })

  it('reads the stream frame as the task and the key', () => {
    const frame = DecisionStreamEventSchema.parse({ taskId: '22222222-2222-2222-2222-222222222222', key: 'region' })

    expect(frame).toEqual({ taskId: '22222222-2222-2222-2222-222222222222', key: 'region' })
    expect(DecisionStreamEventSchema.safeParse({ taskId: 'x' }).success).toBe(false)
  })
})
