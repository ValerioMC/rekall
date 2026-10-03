import { describe, expect, it } from 'vitest'
import { nothingToRun } from '@/model/nothing-to-run'
import type { TaskStepState } from '@/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'

const TASK = 't-1' as TaskId
const OTHER = 't-2' as TaskId
const STEP = 's-1' as TaskStepId

const task = (reviewActive: boolean, reviewState: TaskStepState = 'OPEN') => ({ id: TASK, reviewActive, reviewState })
const step = (state: TaskStepState, taskId: TaskId = TASK) => ({ taskId, state })

describe('a launch with nothing to run', () => {
  it('is empty-handed when every checklist step is claimed or done and one is a draft', () => {
    const steps = [step('CLAIMED'), step('DONE'), step('DRAFT')]

    expect(nothingToRun(task(false), steps, null)).toBe(true)
  })

  it('is empty-handed when a task with no checklist is claimed and only a draft is left', () => {
    expect(nothingToRun(task(true, 'CLAIMED'), [step('DRAFT')], null)).toBe(true)
  })

  it('has work while a step is open or running', () => {
    expect(nothingToRun(task(false), [step('CLAIMED'), step('OPEN'), step('DRAFT')], null)).toBe(false)
    expect(nothingToRun(task(false), [step('RUNNING')], null)).toBe(false)
  })

  it('has work in a task with no checklist whose review is not claimed, drafts or not', () => {
    expect(nothingToRun(task(false), [step('DRAFT')], null)).toBe(false)
    expect(nothingToRun(task(true, 'OPEN'), [step('DRAFT')], null)).toBe(false)
    expect(nothingToRun(task(true, 'CLAIMED'), [], null)).toBe(false)
  })

  it('never refuses a launch on one chosen step', () => {
    expect(nothingToRun(task(false), [step('CLAIMED'), step('DRAFT')], STEP)).toBe(false)
  })

  it('ignores the steps of other tasks and a task it cannot find', () => {
    expect(nothingToRun(task(false), [step('CLAIMED', OTHER)], null)).toBe(false)
    expect(nothingToRun(null, [step('CLAIMED')], null)).toBe(false)
  })
})
