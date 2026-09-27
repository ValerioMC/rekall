import { describe, expect, it } from 'vitest'
import { sessionAtWork } from '@/model/session-at-work'
import type { TaskStepState } from '@/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'

const TASK = 't-1' as TaskId
const OTHER = 't-2' as TaskId
const FIRST = 's-1' as TaskStepId
const SECOND = 's-2' as TaskStepId

const task = (reviewActive: boolean, reviewState: TaskStepState = 'OPEN') => ({ id: TASK, reviewActive, reviewState })
const step = (id: TaskStepId, state: TaskStepState, taskId: TaskId = TASK) => ({ id, taskId, state })
const live = (taskId: TaskId = TASK) => ({ taskId, live: true })

/** When a "Run here" button counts its session as still at work, read from live server state. */
describe('a session at work', () => {
  it('works a step while that step is running, and not a sibling of it', () => {
    const steps = [step(FIRST, 'RUNNING'), step(SECOND, 'OPEN')]

    expect(sessionAtWork(task(false), steps, [live()], FIRST)).toBe(true)
    expect(sessionAtWork(task(false), steps, [live()], SECOND)).toBe(false)
  })

  it('is over for a step once it is claimed, whatever the terminal is doing', () => {
    expect(sessionAtWork(task(false), [step(FIRST, 'CLAIMED')], [live()], FIRST)).toBe(false)
  })

  it('follows the review line of a task with no checklist', () => {
    expect(sessionAtWork(task(true, 'RUNNING'), [], [live()], null)).toBe(true)
    expect(sessionAtWork(task(true, 'CLAIMED'), [], [live()], null)).toBe(false)
  })

  it('works a checklist task while one of its steps runs, even with no terminal known', () => {
    expect(sessionAtWork(task(false), [step(FIRST, 'RUNNING')], [], null)).toBe(true)
  })

  it('works a checklist task while its terminal is live and a step is still open', () => {
    const steps = [step(FIRST, 'CLAIMED'), step(SECOND, 'OPEN')]

    expect(sessionAtWork(task(false), steps, [live()], null)).toBe(true)
    expect(sessionAtWork(task(false), steps, [live(OTHER)], null)).toBe(false)
    expect(sessionAtWork(task(false), steps, [{ taskId: TASK, live: false }], null)).toBe(false)
  })

  it('is over for a checklist task once every step is claimed or done', () => {
    const steps = [step(FIRST, 'CLAIMED'), step(SECOND, 'DONE')]

    expect(sessionAtWork(task(false), steps, [live()], null)).toBe(false)
  })

  it('knows nothing of a task it cannot find', () => {
    expect(sessionAtWork(null, [step(FIRST, 'RUNNING')], [live()], FIRST)).toBe(false)
  })
})
