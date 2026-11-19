import { describe, expect, it } from 'vitest'
import { groupBySession, opensSession, sessionNote, sessionStanding, visibleSessions } from '@/features/steps/lib/step-sessions'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, startsSession: boolean, state: TaskStepState = 'OPEN'): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, startsSession, state, done: state === 'DONE' } as TaskStep
}

describe('groupBySession', () => {
  it('keeps a checklist with no new session in one group', () => {
    const groups = groupBySession([step('a', false), step('b', false)])

    expect(groups).toHaveLength(1)
    expect(groups[0]!.steps.map((member) => member.id)).toEqual(['a', 'b'])
  })

  it('cuts the list at every step that starts a session and numbers the groups from one', () => {
    const groups = groupBySession([step('a', false), step('b', false), step('c', true), step('d', false), step('e', true)])

    expect(groups.map((group) => [group.number, group.steps.map((member) => member.id)])).toEqual([
      [1, ['a', 'b']],
      [2, ['c', 'd']],
      [3, ['e']]
    ])
  })

  it('lets the first step open the first session whatever it says', () => {
    const groups = groupBySession([step('a', true), step('b', false)])

    expect(groups).toHaveLength(1)
    expect(opensSession([step('a', true)], 0)).toBe(false)
  })

  it('has no groups for no steps', () => {
    expect(groupBySession([])).toEqual([])
  })
})

describe('sessionStanding and sessionNote', () => {
  it('calls a session accepted once every step is done, and says how many it held', () => {
    const steps = [step('a', false, 'DONE'), step('b', false, 'DONE')]

    expect(sessionStanding(steps)).toBe('accepted')
    expect(sessionNote(steps)).toBe('Accepted · 2 steps')
  })

  it('calls a session with a step running working, whatever else is claimed in it', () => {
    const steps = [step('a', false, 'CLAIMED'), step('b', false, 'CLAIMED'), step('c', false, 'RUNNING')]

    expect(sessionStanding(steps)).toBe('working')
    expect(sessionNote(steps)).toBe('Working now · 2 of 3 claimed')
  })

  it('calls a session waiting when claims are what is left of it', () => {
    const steps = [step('a', false, 'DONE'), step('b', false, 'CLAIMED')]

    expect(sessionStanding(steps)).toBe('waiting')
    expect(sessionNote(steps)).toBe('Waiting for you · 1 claimed')
  })

  it('calls a session with nothing started not started', () => {
    expect(sessionStanding([step('a', false)])).toBe('idle')
    expect(sessionNote([step('a', false)])).toBe('Not started · 1 step')
  })
})

describe('visibleSessions', () => {
  const sessions = groupBySession([step('a', false, 'DONE'), step('b', true, 'DONE'), step('c', false)])

  it('shows every session while the done steps are shown', () => {
    expect(visibleSessions(sessions, false)).toHaveLength(2)
  })

  it('drops the done steps and the sessions they leave empty, keeping the numbers the plan gave', () => {
    const shown = visibleSessions(sessions, true)

    expect(shown.map((session) => [session.number, session.steps.map((member) => member.id)])).toEqual([[2, ['c']]])
  })
})
