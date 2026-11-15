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

describe('PlanStepRow', () => {
  it('anchors the seal hit area to the seal so it cannot cover the rest of the plan', () => {
    const seal = mount(PlanStepRow, {
      props: { step, number: 1, selected: false, isNext: false, allDone: true, isLast: true, canChooseSession: false }
    }).get('[data-testid="plan-step-seal"]')

    expect(seal.classes()).toContain('hit-target')
    expect(seal.classes()).toContain('relative')
  })
})

describe('PlanStepRow standing and passes', () => {
  const props = { number: 2, selected: false, isNext: false, allDone: false, isLast: false, canChooseSession: false }

  it('says where the step stands under its title and draws a bar for each pass', () => {
    const claimed: TaskStep = {
      ...step,
      state: 'CLAIMED',
      done: false,
      doneAt: null,
      claimedAt: new Date(Date.now() - 14 * 60_000).toISOString(),
      passes: [{ detailMarkdown: null, sentBackAt: '2026-10-01T08:00:00Z' }]
    }

    const row = mount(PlanStepRow, { props: { ...props, step: claimed } })

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
