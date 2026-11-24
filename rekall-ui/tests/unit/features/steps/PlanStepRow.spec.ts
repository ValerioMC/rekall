import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PlanStepRow from '@/features/steps/components/PlanStepRow.vue'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

const step: TaskStep = {
  id: 'step-1' as TaskStepId,
  taskId: 'task-1' as TaskId,
  title: 'Seed the demo world',
  bodyMarkdown: null,
  state: 'DONE',
  done: true,
  runningAt: null,
  claimedAt: null,
  doneAt: '2026-10-01T08:00:00Z',
  position: 0,
  flowX: null,
  flowY: null,
  maxVisits: null,
  startsSession: false,
  connectionId: null,
  passes: [],
  links: [],
  visits: [],
  createdAt: '2026-10-01T07:00:00Z',
  updatedAt: '2026-10-01T07:00:00Z'
}

const places = { canMoveUp: true, canMoveDown: true, sessions: [], aloneBeforeSession: null, aloneAtEnd: true }
const rowProps = { number: 2, selected: false, isNext: false, allDone: false, endsSession: false, droppable: false, dropEdge: null, dragging: false, places }
const open: TaskStep = { ...step, state: 'OPEN', done: false, doneAt: null }
const claimed: TaskStep = { ...step, state: 'CLAIMED', done: false, doneAt: null, claimedAt: new Date(Date.now() - 14 * 60_000).toISOString() }

describe('PlanStepRow', () => {
  it('anchors the seal hit area to the seal so it cannot cover the rest of the plan', () => {
    const seal = mount(PlanStepRow, { props: { ...rowProps, step, number: 1, allDone: true } }).get('[data-testid="plan-step-seal"]')

    expect(seal.classes()).toContain('hit-target')
    expect(seal.classes()).toContain('relative')
  })
})

describe('PlanStepRow standing and passes', () => {
  const props = rowProps

  it('says where the step stands under its title and draws a bar for each pass', () => {
    const reviewed: TaskStep = { ...claimed, passes: [{ detailMarkdown: null, sentBackAt: '2026-10-01T08:00:00Z' }] }

    const row = mount(PlanStepRow, { props: { ...props, step: reviewed } })

    expect(row.get('[data-testid="plan-step-standing"]').text()).toBe('Claimed 14 min ago')
    expect(row.findAll('[data-testid="step-pass-bar"]').map((bar) => bar.attributes('data-kind'))).toEqual(['sent-back', 'in-hand'])
  })

  it('says a running step is being worked and an untouched one is open, with no bars', () => {
    const running = mount(PlanStepRow, { props: { ...props, step: { ...step, state: 'RUNNING', done: false } } })
    const open = mount(PlanStepRow, { props: { ...props, step: { ...step, state: 'OPEN', done: false } } })

    expect(running.get('[data-testid="plan-step-standing"]').text()).toBe('Working now')
    expect(open.get('[data-testid="plan-step-standing"]').text()).toBe('Open')
    expect(open.find('[data-testid="step-pass-bar"]').exists()).toBe(false)
  })
})

describe('PlanStepRow decisions and arranging', () => {
  it('says under the seal what pressing it does, for an open step and for a claim', () => {
    const ticking = mount(PlanStepRow, { props: { ...rowProps, step: open, number: 4 } })
    const accepting = mount(PlanStepRow, { props: { ...rowProps, step: claimed, number: 3 } })

    expect(ticking.get('[data-testid="plan-step-seal-hint"]').text()).toBe('Mark step 4 done')
    expect(accepting.get('[data-testid="plan-step-seal-hint"]').text()).toBe('Accept step 3')
  })

  it('has no hint under the seal of a step that is working or done', () => {
    const running = mount(PlanStepRow, { props: { ...rowProps, step: { ...open, state: 'RUNNING' } } })
    const done = mount(PlanStepRow, { props: { ...rowProps, step } })

    expect(running.find('[data-testid="plan-step-seal-hint"]').exists()).toBe(false)
    expect(done.find('[data-testid="plan-step-seal-hint"]').exists()).toBe(false)
  })

  it('gives an open step a handle and a menu, and a step that has been started neither', () => {
    const arrangeable = mount(PlanStepRow, { props: { ...rowProps, step: open } })
    const started = mount(PlanStepRow, { props: { ...rowProps, step: claimed } })

    expect(arrangeable.find('[data-testid="step-handle"]').exists()).toBe(true)
    expect(arrangeable.find('[data-testid="step-more"]').exists()).toBe(true)
    expect(started.find('[data-testid="step-handle"]').exists()).toBe(false)
    expect(started.find('[data-testid="step-more"]').exists()).toBe(false)
  })

  it('says in words what each menu entry does, and sends the choice up', async () => {
    const row = mount(PlanStepRow, { props: { ...rowProps, step: open } })

    await row.get('[data-testid="step-more"]').trigger('click')
    const hints = row.findAll('[data-testid="popover-menu-hint"]').map((hint) => hint.text())
    expect(hints).toEqual(['Takes it off the plan until you promote it again.', 'Removes it for good. Asks first.'])

    const labelled = (label: string) => row.findAll('[data-testid="popover-menu-item"]').find((item) => item.text().includes(label))!
    await labelled('Return to draft').trigger('click')
    await row.get('[data-testid="step-more"]').trigger('click')
    await labelled('Delete step').trigger('click')
    expect(row.emitted('to-draft')).toHaveLength(1)
    expect(row.emitted('remove')).toHaveLength(1)
  })

  it('offers the sessions the step can run in, each saying why it cannot take it, and sends the choice up', async () => {
    const sessions = [
      { number: 1, current: false, takes: false },
      { number: 2, current: true, takes: true },
      { number: 3, current: false, takes: true }
    ]
    const row = mount(PlanStepRow, {
      props: { ...rowProps, step: open, places: { ...places, sessions, aloneBeforeSession: 2 } }
    })

    await row.get('[data-testid="step-more"]').trigger('click')
    const labelled = (label: string) => row.findAll('[data-testid="popover-menu-item"]').find((item) => item.text().includes(label))!

    expect(labelled('Session 1').attributes('disabled')).toBeDefined()
    expect(labelled('Session 1').text()).toContain('Started, it takes no more steps')
    expect(labelled('Session 2').attributes('aria-current')).toBe('true')
    expect(labelled('Session 3').text()).toContain('Joins it at the start')

    await labelled('Session 3').trigger('click')
    await row.get('[data-testid="step-more"]').trigger('click')
    await labelled('New session before Session 2').trigger('click')
    await row.get('[data-testid="step-more"]').trigger('click')
    await labelled('New session at the end').trigger('click')
    await row.get('[data-testid="step-more"]').trigger('click')
    await labelled('Move down').trigger('click')

    expect(row.emitted('run-in')).toEqual([[3]])
    expect(row.emitted('alone')).toEqual([['before-own-session'], ['end']])
    expect(row.emitted('nudge')).toEqual([[1]])
  })

  it('moves an open step with Alt and the arrow keys, and leaves a started one where it is', async () => {
    const arrangeable = mount(PlanStepRow, { props: { ...rowProps, step: open } })
    const started = mount(PlanStepRow, { props: { ...rowProps, step: claimed } })

    await arrangeable.trigger('keydown', { key: 'ArrowDown', altKey: true })
    await arrangeable.trigger('keydown', { key: 'ArrowUp', altKey: true })
    await started.trigger('keydown', { key: 'ArrowDown', altKey: true })

    expect(arrangeable.emitted('nudge')).toEqual([[1], [-1]])
    expect(started.emitted('nudge')).toBeUndefined()
  })

  it('draws where a dragged step would land from the edge it is given', () => {
    const before = mount(PlanStepRow, { props: { ...rowProps, step: open, dropEdge: 'before' } })
    const after = mount(PlanStepRow, { props: { ...rowProps, step: open, dropEdge: 'after' } })

    expect(before.classes().join(' ')).toContain('before:bg-session-strong')
    expect(after.classes().join(' ')).toContain('after:bg-session-strong')
  })
})
