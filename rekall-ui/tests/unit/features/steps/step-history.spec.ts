import { describe, expect, it } from 'vitest'
import { historyOf } from '@/features/steps/lib/step-history'
import { step } from '../adherence/fixtures'
import type { TaskStep } from '@/features/steps/model/task-step'

function withMoments(changes: Partial<TaskStep>): TaskStep {
  return { ...step(1, 'DONE'), createdAt: '2026-10-01T07:00:00Z', runningAt: null, claimedAt: null, doneAt: null, ...changes } as TaskStep
}

describe('the history of a step', () => {
  it('starts with its creation, which every step has', () => {
    expect(historyOf(withMoments({})).map((event) => event.label)).toEqual(['Created'])
  })

  it('lists each send-back, the claim and the acceptance in the order they happened', () => {
    const accepted = withMoments({
      passes: [
        { detailMarkdown: null, sentBackAt: '2026-10-01T10:00:00Z' },
        { detailMarkdown: null, sentBackAt: '2026-10-01T12:00:00Z' }
      ],
      claimedAt: '2026-10-01T13:00:00Z',
      doneAt: '2026-10-01T14:00:00Z'
    })

    expect(historyOf(accepted).map((event) => [event.kind, event.label])).toEqual([
      ['created', 'Created'],
      ['sent-back', 'Pass 1 sent back'],
      ['sent-back', 'Pass 2 sent back'],
      ['claimed', 'Claimed'],
      ['done', 'Accepted']
    ])
  })

  it('names a session at work', () => {
    const running = withMoments({ runningAt: '2026-10-01T09:00:00Z' })

    expect(historyOf(running).map((event) => event.kind)).toEqual(['created', 'running'])
  })
})
