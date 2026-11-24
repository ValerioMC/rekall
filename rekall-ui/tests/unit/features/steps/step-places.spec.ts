import { describe, expect, it } from 'vitest'
import { placesOf, sessionPlaces } from '@/features/steps/lib/step-places'
import { groupBySession } from '@/features/steps/lib/step-sessions'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, startsSession: boolean, state: TaskStepState = 'OPEN'): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, state, startsSession } as TaskStep
}

describe('placesOf', () => {
  const steps = [step('a', false, 'CLAIMED'), step('b', true), step('c', false), step('d', true)]
  const sessions = groupBySession(steps)

  it('lets a step move only past open neighbours', () => {
    expect(placesOf(steps, sessions, steps[1]!)).toMatchObject({ canMoveUp: false, canMoveDown: true })
    expect(placesOf(steps, sessions, steps[3]!)).toMatchObject({ canMoveUp: true, canMoveDown: false })
  })

  it('marks the session the step runs in, and the ones that have started as unable to take it', () => {
    expect(placesOf(steps, sessions, steps[2]!).sessions).toEqual([
      { number: 1, current: false, takes: false },
      { number: 2, current: true, takes: true },
      { number: 3, current: false, takes: true }
    ])
  })

  it('offers a session of its own in front of its session, or at the end, only where that changes something', () => {
    expect(placesOf(steps, sessions, steps[2]!)).toMatchObject({ aloneBeforeSession: 2, aloneAtEnd: true })
    expect(placesOf(steps, sessions, steps[3]!)).toMatchObject({ aloneBeforeSession: null, aloneAtEnd: false })
  })
})

describe('sessionPlaces', () => {
  it('describes the sessions a draft could be promoted into, none of which it is in', () => {
    const sessions = groupBySession([step('a', false, 'CLAIMED'), step('b', true)])

    expect(sessionPlaces(sessions, null)).toEqual([
      { number: 1, current: false, takes: false },
      { number: 2, current: false, takes: true }
    ])
  })
})
