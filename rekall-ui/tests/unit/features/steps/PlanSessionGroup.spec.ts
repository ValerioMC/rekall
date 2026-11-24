import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PlanSessionGroup from '@/features/steps/components/PlanSessionGroup.vue'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, state: TaskStepState, startsSession = false): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, title: `Step ${id}`, state, startsSession, done: state === 'DONE' } as TaskStep
}

function mountGroup(session: StepSession, holdsSelection = false, workRemains = true, movable = false, foldedForDrag = false) {
  return mount(PlanSessionGroup, {
    props: { session, holdsSelection, workRemains, movable, foldedForDrag },
    slots: { step: '<li data-testid="row">{{ params.step.title }}</li>' }
  })
}

const working: StepSession = { number: 2, steps: [step('a', 'CLAIMED', true), step('b', 'CLAIMED'), step('c', 'RUNNING')] }

describe('PlanSessionGroup', () => {
  it('says where the session stands and keeps its steps in view', () => {
    const group = mountGroup(working)

    expect(group.get('[data-testid="plan-session"]').attributes('data-standing')).toBe('working')
    expect(group.get('[data-testid="session-note"]').text()).toBe('Working now · 2 of 3 claimed')
    expect(group.findAll('[data-testid="row"]')).toHaveLength(3)
  })

  it('offers a cut between two steps that can still change session, and none before one that was handed in', () => {
    const group = mountGroup({ number: 2, steps: [step('a', 'OPEN', true), step('b', 'OPEN'), step('c', 'CLAIMED')] })

    expect(group.findAll('[data-testid="step-new-session"]')).toHaveLength(1)
  })

  it('sends the step to cut the session at', async () => {
    const group = mountGroup({ number: 2, steps: [step('a', 'OPEN', true), step('b', 'OPEN')] })

    await group.get('[data-testid="step-new-session"]').trigger('click')

    expect(group.emitted('split')?.[0]).toEqual([expect.objectContaining({ id: 'b' })])
  })

  it('offers to join the session above only when it is not the first, opens at a step not handed in, and is not started', async () => {
    const idle: StepSession = { number: 2, steps: [step('a', 'OPEN', true), step('b', 'OPEN')] }

    const group = mountGroup(idle)
    await group.get('[data-testid="session-join"]').trigger('click')

    expect(group.emitted('join')).toHaveLength(1)
    expect(mountGroup({ ...idle, number: 1 }).find('[data-testid="session-join"]').exists()).toBe(false)
    expect(mountGroup(working).find('[data-testid="session-join"]').exists()).toBe(false)
  })

  it('folds an accepted session to its header and opens it again from there', async () => {
    const accepted: StepSession = { number: 1, steps: [step('a', 'DONE'), step('b', 'DONE')] }
    const group = mountGroup(accepted)

    expect(group.find('[data-testid="session-steps"]').exists()).toBe(false)
    await group.get('[data-testid="session-header"]').trigger('click')

    expect(group.findAll('[data-testid="row"]')).toHaveLength(2)
  })

  it('keeps every session open once the whole plan is accepted, so a step can still be reopened', () => {
    const accepted: StepSession = { number: 1, steps: [step('a', 'DONE')] }

    expect(mountGroup(accepted, false, false).find('[data-testid="session-steps"]').exists()).toBe(true)
  })

  it('keeps an accepted session open while one of its steps is selected', () => {
    const accepted: StepSession = { number: 1, steps: [step('a', 'DONE')] }

    expect(mountGroup(accepted, true).find('[data-testid="session-steps"]').exists()).toBe(true)
  })

  it('offers the grip only to a session that can be lifted whole, and sends the drag from it', async () => {
    const idle: StepSession = { number: 2, steps: [step('a', 'OPEN', true), step('b', 'OPEN')] }

    expect(mountGroup(idle).find('[data-testid="session-handle"]').exists()).toBe(false)
    const group = mountGroup(idle, false, true, true)
    await group.get('[data-testid="session-handle"]').trigger('pointerdown')
    await group.get('[data-testid="session-header"]').element.parentElement!.dispatchEvent(
      Object.assign(new Event('dragstart', { bubbles: true, cancelable: true }), { dataTransfer: { setData: () => undefined } })
    )

    expect(group.emitted('drag-start')).toHaveLength(1)
  })

  it('moves the whole session one place on Alt and the arrow keys from its grip', async () => {
    const group = mountGroup({ number: 2, steps: [step('a', 'OPEN', true)] }, false, true, true)

    await group.get('[data-testid="session-handle"]').trigger('keydown', { key: 'ArrowDown', altKey: true })

    expect(group.emitted('nudge')?.[0]).toEqual([1])
  })

  it('folds to its header while a session is held, and says a started one stays where it is', () => {
    const started = mountGroup(working, false, true, false, true)

    expect(started.find('[data-testid="session-steps"]').exists()).toBe(false)
    expect(started.get('[data-testid="session-started"]').text()).toBe('Started')
  })
})
