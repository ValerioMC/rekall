import { describe, expect, it } from 'vitest'
import {
  layoutOf,
  openerHandover,
  planRelayout,
  planSessionMove,
  planStepAlone,
  planStepMove,
  sessionIsMovable,
  sessionLanding,
  type Relayout
} from '@/features/steps/lib/step-order'
import { groupBySession } from '@/features/steps/lib/step-sessions'
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

function applied(steps: readonly TaskStep[], plan: Relayout | null): string | null {
  if (!plan) return null
  const order = [...steps]
  plan.moves.forEach(({ step, position }) => {
    order.splice(order.indexOf(step), 1)
    order.splice(position, 0, step)
  })
  const flags = new Map<string, boolean>(steps.map((each) => [each.id, each.startsSession]))
  plan.sessionChanges.forEach((change) => flags.set(change.step.id, change.startsSession))
  return order.map((each, index) => (index > 0 && flags.get(each.id) ? `|${each.id}` : each.id)).join(' ')
}

describe('planStepAlone', () => {
  const checklist = (): TaskStep[] => [
    step('a', false, 'CLAIMED'),
    step('b', true),
    step('c', false),
    step('d', false),
    step('e', true),
    step('f', false)
  ]

  it('opens a session for the step in front of the one it goes before, which keeps opening its own', () => {
    const steps = checklist()

    expect(applied(steps, planStepAlone(steps, steps[3]!, steps[4]!))).toBe('a |b c |d |e f')
  })

  it('splits the session and hands the opening on when the step leaves the head of a session', () => {
    const steps = checklist()

    expect(applied(steps, planStepAlone(steps, steps[1]!, steps[4]!))).toBe('a |c d |b |e f')
  })

  it('runs the step last in a session of its own when no step is named, with one lift', () => {
    const steps = checklist()
    const plan = planStepAlone(steps, steps[2]!, null)

    expect(applied(steps, plan)).toBe('a |b d |e f |c')
    expect(plan?.moves).toHaveLength(1)
  })

  it('splits its own session when put in front of itself, and does nothing when it is alone already', () => {
    const steps = checklist()

    expect(applied(steps, planStepAlone(steps, steps[1]!, steps[1]!))).toBe('a |b |c d |e f')
    expect(planStepAlone([step('a', false), step('b', true), step('c', true)], steps[1]!, steps[1]!)).toBeNull()
  })

  it('never lands above a step that has been started', () => {
    const steps = checklist()

    expect(planStepAlone(steps, steps[3]!, steps[0]!)).toBeNull()
  })
})

describe('planSessionMove', () => {
  const checklist = (): TaskStep[] => [
    step('a', false, 'CLAIMED'),
    step('b', true),
    step('c', false),
    step('d', true),
    step('e', false)
  ]

  it('carries every step of the session in order and keeps the sessions it passes intact', () => {
    const steps = checklist()
    const [, second, third] = groupBySession(steps)

    expect(applied(steps, planSessionMove(steps, third!, second!.steps[0]!))).toBe('a |d e |b c')
  })

  it('moves a session to the end behind the one after it', () => {
    const steps = checklist()
    const [, second] = groupBySession(steps)

    expect(applied(steps, planSessionMove(steps, second!, null))).toBe('a |d e |b c')
  })

  it('does not move a session to where it already is, nor above a session that has started', () => {
    const steps = checklist()
    const [first, second, third] = groupBySession(steps)

    expect(planSessionMove(steps, second!, third!.steps[0]!)).toBeNull()
    expect(planSessionMove(steps, third!, null)).toBeNull()
    expect(planSessionMove(steps, third!, first!.steps[0]!)).toBeNull()
  })
})

describe('planRelayout', () => {
  it('turns the checklist into a layout it was before, which is how an undo works', () => {
    const steps = [step('a', false), step('b', true), step('c', false)]
    const before = layoutOf(steps)
    const after = [step('a', false), step('c', true), step('b', false)]

    expect(applied(after, planRelayout(after, before))).toBe('a |b c')
  })

  it('has nothing to do when the layout is already the one asked for, whatever the first step says', () => {
    const steps = [step('a', true), step('b', true)]

    expect(planRelayout(steps, [{ id: steps[0]!.id, startsSession: false }, { id: steps[1]!.id, startsSession: true }])).toBeNull()
  })
})

describe('sessionIsMovable and sessionLanding', () => {
  it('lifts a session only while all its steps are open', () => {
    const [claimed, idle] = groupBySession([step('a', false, 'CLAIMED'), step('b', true), step('c', false)])

    expect(sessionIsMovable(claimed!)).toBe(false)
    expect(sessionIsMovable(idle!)).toBe(true)
  })

  it('lands behind the last step of an earlier session and at the head of a later one', () => {
    const steps = [step('a', false), step('b', false), step('c', true), step('d', false), step('e', true)]
    const sessions = groupBySession(steps)

    expect(sessionLanding(sessions, steps[3]!, sessions[0]!)).toEqual({ target: steps[1], after: true })
    expect(sessionLanding(sessions, steps[3]!, sessions[2]!)).toEqual({ target: steps[4], after: false })
  })
})
