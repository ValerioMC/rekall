import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import PlanBar from '@/features/steps/components/PlanBar.vue'
import * as api from '@/features/adherence/api/adherence.api'
import { TASK, adherence, step } from '../adherence/fixtures'

vi.mock('@/features/adherence/api/adherence.api')

async function render(props: Record<string, unknown> = {}) {
  const wrapper = mount(PlanBar, {
    props: {
      taskId: TASK,
      steps: [step(1, 'DONE'), step(2, 'DONE', 1), step(3, 'DONE')],
      done: 3,
      claimed: 0,
      total: 3,
      runningNumber: null,
      draftCount: 0,
      canShowFlow: true,
      listKeys: true,
      hideDone: false,
      view: 'list',
      ...props
    },
    attachTo: document.body
  })
  await flushPromises()
  return wrapper
}

describe('the plan bar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence())
    document.body.innerHTML = ''
  })

  it('shows the adherence figures on its one row and keeps the full panel closed', async () => {
    const bar = await render()

    const figures = bar.get('[data-testid="adherence-figures"]').text()
    expect(figures).toContain('2/3 first time')
    expect(figures).toContain('1 sent back')
    expect(figures).toContain('1 deviation')
    expect(bar.find('[data-testid="plan-adherence"]').exists()).toBe(false)
  })

  it('opens the full panel over the workspace from the figures and closes it on Escape', async () => {
    const bar = await render()

    await bar.get('[data-testid="adherence-figures"]').trigger('click')
    expect(bar.get('[data-testid="plan-adherence"]').text()).toContain('Plan adherence')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    expect(bar.find('[data-testid="plan-adherence"]').exists()).toBe(false)
  })

  it('names the budgets spent in the figures', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(
      adherence({ visits: [{ stepNumber: 2, title: 'Fix', visits: 3, maxVisits: 3, budgetSpent: true }] })
    )

    const bar = await render()

    expect(bar.get('[data-testid="adherence-budgets-spent-chip"]').text()).toBe('1 budget spent')
  })

  it('offers Hide done only while the list is shown', async () => {
    const list = await render()
    const flow = await render({ view: 'flow', listKeys: false })

    expect(list.find('[data-testid="toggle-hide-done"]').exists()).toBe(true)
    expect(flow.find('[data-testid="toggle-hide-done"]').exists()).toBe(false)
  })

  it('offers Review next while a step waits, and the flow view only for a flow', async () => {
    const waiting = await render({ claimed: 1, done: 2, canShowFlow: false })

    expect(waiting.find('[data-testid="plan-review-next"]').exists()).toBe(true)
    expect(waiting.find('[data-testid="segment-flow"]').exists()).toBe(false)
    expect((await render()).find('[data-testid="plan-review-next"]').exists()).toBe(false)
  })
})
