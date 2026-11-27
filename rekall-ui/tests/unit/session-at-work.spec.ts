import { describe, expect, it } from 'vitest'
import { sessionAtWork } from '@/model/session-at-work'
import type { TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

const TASK = 't-1' as TaskId
const OTHER = 't-2' as TaskId
const FIRST = 's-1' as TaskStepId
const SECOND = 's-2' as TaskStepId

const task = (reviewActive: boolean, reviewState: TaskStepState = 'OPEN') => ({ id: TASK, reviewActive, reviewState })
const step = (id: TaskStepId, state: TaskStepState, taskId: TaskId = TASK) => ({ id, taskId, state })
const live = (taskId: TaskId = TASK) => ({ taskId, live: true, mode: 'WORK' as const })
const discussion = () => ({ ...live(), mode: 'DISCUSS' as const })

/** When the "Run here" key counts the task's session as still at work, read from live server state. */
describe('a session at work', () => {
  it('follows the review line of a task with no checklist', () => {
    expect(sessionAtWork(task(true, 'RUNNING'), [], [live()])).toBe(true)
    expect(sessionAtWork(task(true, 'CLAIMED'), [], [live()])).toBe(false)
  })

  it('works a checklist task while one of its steps runs, even with no terminal known', () => {
    expect(sessionAtWork(task(false), [step(FIRST, 'RUNNING')], [])).toBe(true)
  })

  it('works a checklist task while its terminal is live and a step is still open', () => {
    const steps = [step(FIRST, 'CLAIMED'), step(SECOND, 'OPEN')]

    expect(sessionAtWork(task(false), steps, [live()])).toBe(true)
    expect(sessionAtWork(task(false), steps, [live(OTHER)])).toBe(false)
    expect(sessionAtWork(task(false), steps, [{ ...live(), live: false }])).toBe(false)
  })

  it('does not count a discussion as work, so the open steps can still be run', () => {
    expect(sessionAtWork(task(false), [step(FIRST, 'OPEN')], [discussion()])).toBe(false)
  })

  it('is over for a checklist task once every step is claimed or done', () => {
    const steps = [step(FIRST, 'CLAIMED'), step(SECOND, 'DONE')]

    expect(sessionAtWork(task(false), steps, [live()])).toBe(false)
  })

  it('knows nothing of a task it cannot find', () => {
    expect(sessionAtWork(null, [step(FIRST, 'RUNNING')], [live()])).toBe(false)
  })
})
