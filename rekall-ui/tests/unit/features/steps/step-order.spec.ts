import { describe, expect, it } from 'vitest'
import { openerHandover, planStepMove } from '@/features/steps/lib/step-order'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, startsSession: boolean, state: TaskStepState = 'OPEN'): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, state, startsSession } as TaskStep
}

function changes(plan: ReturnType<typeof planStepMove>): [string, boolean][] {
  return (plan?.sessionChanges ?? []).map((change) => [change.step.id, change.startsSession])
}

describe('openerHandover', () => {
  it('hands the opening to the step behind an opener that leaves', () => {
    const steps = [step('a', false), step('b', true), step('c', false)]

    expect(openerHandover(steps, steps[1]!).map((change) => [change.step.id, change.startsSession])).toEqual([['c', true]])
  })

  it('hands nothing over when the step opens no session, the next one already does, or none follows', () => {
    const steps = [step('a', false), step('b', false), step('c', true), step('d', true)]

    expect(openerHandover(steps, steps[1]!)).toEqual([])
    expect(openerHandover(steps, steps[2]!)).toEqual([])
    expect(openerHandover(steps, steps[3]!)).toEqual([])
  })

  it('hands nothing over from the first step, which opens the first session whatever it says', () => {
    const steps = [step('a', true), step('b', false)]

    expect(openerHandover(steps, steps[0]!)).toEqual([])
  })
})

describe('planStepMove', () => {
  const checklist = (): TaskStep[] => [step('a', false, 'DONE'), step('b', true), step('c', false), step('d', false)]

  it('joins the session the step lands in when it goes behind a step', () => {
    const steps = checklist()

    const plan = planStepMove(steps, steps[3]!, steps[1]!, true)

    expect(plan?.position).toBe(2)
    expect(changes(plan)).toEqual([])
  })

  it('opens the session the step lands at the head of, taking the opening from the step it goes in front of', () => {
    const steps = checklist()

    const plan = planStepMove(steps, steps[3]!, steps[1]!, false)

    expect(plan?.position).toBe(1)
    expect(changes(plan)).toEqual([
      ['b', false],
      ['d', true]
    ])
  })

  it('keeps the steps behind a session opener in its session when the opener moves away, and the mover joins where it lands', () => {
    const steps = checklist()

    const plan = planStepMove(steps, steps[1]!, steps[3]!, true)

    expect(plan?.position).toBe(3)
    expect(changes(plan)).toEqual([
      ['b', false],
      ['c', true]
    ])
  })

  it('does not move a step to the place it is already in', () => {
    const steps = checklist()

    expect(planStepMove(steps, steps[2]!, steps[1]!, true)).toBeNull()
    expect(planStepMove(steps, steps[2]!, steps[3]!, false)).toBeNull()
    expect(planStepMove(steps, steps[2]!, steps[2]!, true)).toBeNull()
  })

  it('does not mark the first step of the list as opening a session', () => {
    const steps = [step('a', false), step('b', true), step('c', false)]

    const plan = planStepMove(steps, steps[2]!, steps[0]!, false)

    expect(plan?.position).toBe(0)
    expect(changes(plan)).toEqual([])
  })
})
