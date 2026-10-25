import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import StepsPane from '@/features/steps/components/StepsPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskStep } from '@/features/catalog/model/catalog'
import { finishedFlow, flowStep, releaseFlow, releaseTask, TASK, visit } from './flow.fixtures'

/**
 * The steps pane showing a flow: it opens on the graph when the steps carry routes, lights the
 * step the work stands on, counts the moves and visits, follows a session's move without a
 * reload, and sends route edits and the reset through the store.
 */
let pinia: Pinia

function seed(steps: TaskStep[]) {
  const store = useConsoleStore()
  store.tasks = [releaseTask()]
  store.selectedTaskId = TASK
  store.steps = steps
  store.isLoading = false
  store.addRoute = vi.fn().mockResolvedValue(undefined)
  store.editRoute = vi.fn().mockResolvedValue(undefined)
  store.removeRoute = vi.fn().mockResolvedValue(undefined)
  store.resetFlow = vi.fn().mockResolvedValue(undefined)
  store.placeStep = vi.fn().mockResolvedValue(undefined)
  store.tidyFlow = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  return mount(StepsPane, {
    attachTo: document.body,
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        NotesButton: true,
        PlanHereButton: true,
        OpenTerminalButton: true,
        LaunchClaudeCodeButton: true,
        LogCommitButton: true,
        CommitReferenceList: true,
        AppConfirm: {
          template:
            '<div data-testid="confirm"><button data-testid="confirm-yes" @click="$emit(\'confirm\')" /></div>'
        }
      }
    }
  })
}

describe('the flow view', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('opens on the graph when the steps carry routes', () => {
    seed(releaseFlow())
    const pane = render()

    expect(pane.find('[data-testid="step-flow-view"]').exists()).toBe(true)
    expect(pane.findAll('[data-testid="flow-node"]')).toHaveLength(5)
    expect(pane.get('[data-testid="steps-view-flow"]').attributes('aria-pressed')).toBe('true')
  })

  it('stays on the list for a checklist with no routes, and switches on request', async () => {
    seed([flowStep('one', 0), flowStep('two', 1)])
    const pane = render()
    expect(pane.find('[data-testid="step-flow-view"]').exists()).toBe(false)

    await pane.get('[data-testid="steps-view-flow"]').trigger('click')

    expect(pane.find('[data-testid="step-flow-view"]').exists()).toBe(true)
    expect(pane.find('[data-testid="flow-hint"]').text()).toContain('still a list')
  })

  it('lights the step the work stands on and counts every visit', () => {
    seed(releaseFlow())
    const pane = render()

    const current = pane.get('[data-current="true"]')
    expect(current.attributes('data-step-id')).toBe('deploy')
    expect(current.get('[data-testid="flow-node-visits"]').text()).toBe('×2')
    expect(pane.get('[data-step-id="ship"] [data-testid="flow-node-visits"]').text()).toBe('not reached')
  })

  it('shows the moves in order with the outcome each one left by', () => {
    seed(releaseFlow())
    const pane = render()

    expect(pane.get('[data-testid="flow-move-count"]').text()).toContain('5')
    const moves = pane.findAll('[data-testid="flow-move"]')
    expect(moves.map((move) => move.text())).toEqual(['1Build', '2Deploy', '3Test', '4Fix', '5Deploy'])
    expect(moves[4]!.attributes('aria-current')).toBe('step')
    expect(pane.get('[data-testid="flow-trail"]').text()).toContain('app-error ›')
  })

  it('shows on the trail what a move reported when it left its step', () => {
    seed(
      releaseFlow().map((step) =>
        step.id === 'test' ? { ...step, visits: [visit('test', 3, 'app-error', false, 'GET /health answers 500')] } : step
      )
    )
    const pane = render()

    const outcomes = pane.findAll('[data-testid="flow-move-outcome"]')
    expect(outcomes[2]!.attributes('title')).toBe('GET /health answers 500')
    expect(outcomes[0]!.attributes('title')).toBeUndefined()
    expect(pane.findAll('[data-testid="flow-move"]')[2]!.attributes('title')).toBe(
      'Move 3: Test, left by app-error: GET /health answers 500'
    )
  })

  it('follows a move a session makes, with no reload', async () => {
    const store = seed(releaseFlow())
    const pane = render()
    const moved = releaseFlow().map((step) => {
      if (step.id === 'deploy')
        return {
          ...step,
          state: 'CLAIMED' as const,
          visits: step.visits.map((v) => ({
            ...v,
            leftAt: v.leftAt ?? '2026-10-01T09:06:00Z',
            outcome: v.outcome ?? 'deployed'
          }))
        }
      if (step.id === 'test')
        return {
          ...step,
          state: 'OPEN' as const,
          visits: [
            ...step.visits,
            {
              id: 'test-6' as never,
              stepId: step.id,
              ordinal: 6,
              enteredAt: '2026-10-01T09:06:00Z',
              leftAt: null,
              outcome: null,
              report: null
            }
          ]
        }
      return step
    })

    store.applyStepEvent(TASK, moved)
    await flushPromises()

    expect(pane.get('[data-current="true"]').attributes('data-step-id')).toBe('test')
    expect(pane.findAll('[data-testid="flow-move"]')).toHaveLength(6)
  })

  it('adds a route from the picked step through the store', async () => {
    const store = seed(releaseFlow())
    const pane = render()
    await pane.get('[data-step-id="ship"]').trigger('click')

    const form = pane.get('[data-testid="flow-route-add"]')
    await form.get('input').setValue('rollback')
    await form.get('select').setValue('deploy')
    await form.trigger('submit')
    await flushPromises()

    expect(store.addRoute).toHaveBeenCalledWith(TASK, 'ship', 'deploy', 'rollback')
  })

  it('removes a route and resets the moves once confirmed', async () => {
    const store = seed(releaseFlow())
    const pane = render()
    await pane.get('[data-step-id="test"]').trigger('click')

    await pane.get('[data-testid="flow-route-row"] button').trigger('click')
    await pane.get('[data-testid="flow-reset"]').trigger('click')
    await pane.get('[data-testid="confirm-yes"]').trigger('click')
    await flushPromises()

    expect(store.removeRoute).toHaveBeenCalledWith(TASK, 'test-app-error')
    expect(store.resetFlow).toHaveBeenCalledWith(TASK)
  })

  it('opens the picked step on the list when its detail is asked for', async () => {
    seed(releaseFlow())
    const pane = render()
    await pane.get('[data-step-id="test"]').trigger('click')

    await pane.get('[data-testid="flow-open-detail"]').trigger('click')
    await flushPromises()

    expect(pane.find('[data-testid="step-flow-view"]').exists()).toBe(false)
    expect(pane.find('li[data-step-id="test"]').exists()).toBe(true)
  })

  it('always draws Start and End, lit only when the work has been through them', () => {
    seed(releaseFlow())
    const pane = render()

    expect(pane.get('[data-testid="flow-start"]').classes()).toContain('flow-terminal-reached')
    expect(pane.get('[data-testid="flow-end"]').classes()).toContain('flow-terminal-idle')
  })

  it('lights End and leaves no step current once a route to the end was taken', () => {
    seed(finishedFlow())
    const pane = render()

    expect(pane.get('[data-testid="flow-end"]').classes()).toContain('flow-terminal-reached')
    expect(pane.find('[data-current="true"]').exists()).toBe(false)
  })

  it('shows a start and an end even on a plain checklist', async () => {
    seed([flowStep('one', 0), flowStep('two', 1)])
    const pane = render()
    await pane.get('[data-testid="steps-view-flow"]').trigger('click')

    expect(pane.find('[data-testid="flow-start"]').exists()).toBe(true)
    expect(pane.find('[data-testid="flow-end"]').exists()).toBe(true)
  })

  it('adds a route that ends the flow when End is picked as its target', async () => {
    const store = seed(releaseFlow())
    const pane = render()
    await pane.get('[data-step-id="ship"]').trigger('click')

    const form = pane.get('[data-testid="flow-route-add"]')
    await form.get('input').setValue('shipped')
    await form.get('select').setValue('flow:end')
    await form.trigger('submit')
    await flushPromises()

    expect(store.addRoute).toHaveBeenCalledWith(TASK, 'ship', 'flow:end', 'shipped')
  })

  it('saves where a step was nudged to with the keyboard', async () => {
    const store = seed(releaseFlow())
    const pane = render()

    await pane.get('[data-step-id="ship"]').trigger('keydown', { key: 'ArrowRight', altKey: true })
    await flushPromises()

    expect(store.placeStep).toHaveBeenCalledWith(
      TASK,
      'ship',
      expect.objectContaining({ x: expect.any(Number), y: expect.any(Number) })
    )
  })

  it('offers to tidy the layout only once a step has been placed', async () => {
    const store = seed(releaseFlow())
    const pane = render()
    expect(pane.find('[data-testid="flow-tidy"]').exists()).toBe(false)

    store.applyStepEvent(
      TASK,
      releaseFlow().map((step) => (step.id === 'ship' ? { ...step, flowX: 400, flowY: 300 } : step))
    )
    await flushPromises()
    await pane.get('[data-testid="flow-tidy"]').trigger('click')

    expect(store.tidyFlow).toHaveBeenCalledWith(TASK)
  })
})
