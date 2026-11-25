import { describe, expect, it } from 'vitest'
import { stepTally, taskActivity } from '@/features/workingset/lib/task-activity'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, state: TaskStepState): TaskStep {
  return {
    id: id as TaskStepId,
    taskId: 't1' as TaskId,
    title: id,
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: null,
    doneAt: null,
    position: 0,
    flowX: null,
    flowY: null,
    maxVisits: null,
    startsSession: false,
    connectionId: null,
    passes: [],
    links: [],
    visits: [],
    createdAt: '2026-11-25T09:00:00Z',
    updatedAt: '2026-11-25T09:00:00Z'
  }
}

describe('task activity', () => {
  it('is working while a step is running', () => {
    expect(taskActivity({ steps: [step('a', 'RUNNING')], turn: null, hasLiveSession: false })).toBe('WORKING')
  })

  it('is working while the chat turn is working, with no running step', () => {
    expect(taskActivity({ steps: [step('a', 'OPEN')], turn: 'working', hasLiveSession: true })).toBe('WORKING')
  })

  it.each<TaskStepState>(['CLAIMED', 'DONE'])('is executed with a %s step and nothing running', (state) => {
    expect(taskActivity({ steps: [step('a', state)], turn: 'idle', hasLiveSession: false })).toBe('EXECUTED')
  })

  it('is executed with a live session and no complete step', () => {
    expect(taskActivity({ steps: [step('a', 'OPEN')], turn: null, hasLiveSession: true })).toBe('EXECUTED')
  })

  it('is inactive with no steps and no session', () => {
    expect(taskActivity({ steps: [], turn: null, hasLiveSession: false })).toBe('INACTIVE')
  })
})

describe('step tally', () => {
  it('counts claimed and done as executed and leaves drafts out of both numbers', () => {
    const steps = [step('a', 'DONE'), step('b', 'CLAIMED'), step('c', 'RUNNING'), step('d', 'OPEN'), step('e', 'DRAFT')]

    expect(stepTally(steps)).toEqual({ executed: 2, total: 4 })
  })

  it('is zero of zero for a task with no steps', () => {
    expect(stepTally([])).toEqual({ executed: 0, total: 0 })
  })
})
