import { describe, expect, it } from 'vitest'
import { neighbourOf, stepNumberOf } from '@/features/deviations/model/deviation'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'
import { deviation } from './fixtures'

function step(id: string, state: TaskStepState): TaskStep {
  return { id: id as TaskStepId, state } as TaskStep
}

describe('the step a deviation names', () => {
  it('counts the checklist from one and leaves the drafts out, as the context does', () => {
    const steps = [step('draft', 'DRAFT'), step('first', 'DONE'), step('second', 'OPEN')]

    expect(stepNumberOf(steps[1]!, steps)).toBe(1)
    expect(stepNumberOf(steps[2]!, steps)).toBe(2)
    expect(stepNumberOf(steps[0]!, steps)).toBeNull()
  })
})

describe('walking the deviations of a task', () => {
  const deviations = [deviation(2), deviation(5), deviation(9)]

  it('goes to the next and the previous in step order', () => {
    expect(neighbourOf(deviations, 5, 1)?.stepNumber).toBe(9)
    expect(neighbourOf(deviations, 5, -1)?.stepNumber).toBe(2)
  })

  it('stops at either end', () => {
    expect(neighbourOf(deviations, 9, 1)).toBeNull()
    expect(neighbourOf(deviations, 2, -1)).toBeNull()
  })
})
