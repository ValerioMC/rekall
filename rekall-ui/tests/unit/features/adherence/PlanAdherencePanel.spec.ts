import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import PlanAdherencePanel from '@/features/adherence/components/PlanAdherencePanel.vue'
import * as api from '@/features/adherence/api/adherence.api'
import type { TaskStep } from '@/features/steps/model/task-step'
import { TASK, adherence, step } from './fixtures'

vi.mock('@/features/adherence/api/adherence.api')

async function render(steps: TaskStep[]) {
  const wrapper = mount(PlanAdherencePanel, { props: { taskId: TASK, steps } })
  await flushPromises()
  return wrapper
}

describe('the plan adherence panel of a task', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('shows the three numbers and the help text', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence())

    const wrapper = await render([step(1, 'DONE'), step(2, 'DONE', 2), step(3, 'DONE')])

    expect(wrapper.get('[data-testid="adherence-first-time"]').text()).toBe('2 of 3')
    expect(wrapper.text()).toContain('67% of the accepted steps')
    expect(wrapper.get('[data-testid="adherence-sent-back"]').text()).toBe('1')
    expect(wrapper.text()).toContain('5 passes in all')
    expect(wrapper.get('[data-testid="adherence-deviations"]').text()).toBe('1')
    expect(wrapper.text()).toContain('on step 2')
    expect(wrapper.get('[data-testid="adherence-help"]').text()).toBe(
      'A high share of steps sent back usually means steps are too large or their detail too vague.'
    )
  })

  it('draws one segment per pass on each step and says the count in words too', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence())

    const wrapper = await render([step(1, 'DONE'), step(2, 'DONE', 2), step(3, 'DONE')])

    const rows = wrapper.findAll('[data-testid="adherence-step"]')
    expect(rows.map((row) => row.findAll('[data-testid="adherence-pass"]').length)).toEqual([1, 3, 1])
    expect(rows[1]?.get('[data-testid="adherence-step-summary"]').text()).toBe('3 passes, 2 sent back')
    expect(rows[1]?.findAll('[data-kind="sent-back"]')).toHaveLength(2)
  })

  it('leaves drafts out of the steps it draws', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence())

    const wrapper = await render([step(1, 'DONE'), step(2, 'DRAFT')])

    expect(wrapper.findAll('[data-testid="adherence-step"]')).toHaveLength(1)
  })

  it('says there is no plan to measure for a task with no steps', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence({ stepsTotal: 0, stepsDone: 0, passesTotal: 0, acceptedFirstTime: 0, sentBack: 0, firstTimeShare: null, deviations: 0, deviationSteps: [] }))

    const wrapper = await render([])

    expect(wrapper.find('[data-testid="adherence-no-steps"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="adherence-first-time"]').exists()).toBe(false)
  })

  it('says no step has been worked when nothing has a pass', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence({ stepsDone: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0, firstTimeShare: null, deviations: 0, deviationSteps: [] }))

    const wrapper = await render([step(1, 'OPEN'), step(2, 'OPEN')])

    expect(wrapper.find('[data-testid="adherence-no-passes"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="adherence-step"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="adherence-first-time"]').text()).toBe('0 of 0')
    expect(wrapper.text()).toContain('no step accepted yet')
  })

  it('lists the visits of a flow against their budget and names the spent one in words', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(
      adherence({
        visits: [
          { stepNumber: 1, title: 'Build', visits: 1, maxVisits: null, budgetSpent: false },
          { stepNumber: 2, title: 'Fix', visits: 3, maxVisits: 3, budgetSpent: true }
        ]
      })
    )

    const wrapper = await render([step(1, 'DONE'), step(2, 'RUNNING')])

    const rows = wrapper.findAll('[data-testid="adherence-visit"]')
    expect(rows[0]?.text()).toContain('1')
    expect(rows[0]?.find('[data-testid="adherence-budget-spent"]').exists()).toBe(false)
    expect(rows[1]?.text()).toContain('3 of 3')
    expect(rows[1]?.get('[data-testid="adherence-budget-spent"]').text()).toBe('budget spent')
  })

  it('reads again when a step changes state', async () => {
    vi.mocked(api.fetchTaskAdherence).mockResolvedValue(adherence())
    const wrapper = await render([step(1, 'RUNNING')])

    await wrapper.setProps({ steps: [step(1, 'CLAIMED')] })
    await flushPromises()

    expect(api.fetchTaskAdherence).toHaveBeenCalledTimes(2)
  })
})
