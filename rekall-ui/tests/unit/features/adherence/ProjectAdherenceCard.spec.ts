import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ProjectAdherenceCard from '@/features/adherence/components/ProjectAdherenceCard.vue'
import * as api from '@/features/adherence/api/adherence.api'
import { PROJECT, projectAdherence } from './fixtures'

vi.mock('@/features/adherence/api/adherence.api')

async function render() {
  const wrapper = mount(ProjectAdherenceCard, { props: { projectId: PROJECT, taskCount: 2 } })
  await flushPromises()
  return wrapper
}

describe('the plan adherence card of a project', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('shows the numbers summed over the project', async () => {
    vi.mocked(api.fetchProjectAdherence).mockResolvedValue(projectAdherence())

    const wrapper = await render()

    expect(wrapper.get('[data-testid="project-adherence-first-time"]').text()).toBe('1 of 4')
    expect(wrapper.text()).toContain('25% of the accepted steps')
    expect(wrapper.get('[data-testid="project-adherence-sent-back"]').text()).toBe('3')
    expect(wrapper.get('[data-testid="project-adherence-deviations"]').text()).toBe('1')
  })

  it('lists the tasks with most steps sent back and opens the one clicked', async () => {
    vi.mocked(api.fetchProjectAdherence).mockResolvedValue(projectAdherence())
    const wrapper = await render()

    const rows = wrapper.findAll('[data-testid="project-adherence-task"]')
    expect(rows.map((row) => row.text())).toEqual(['Import2 sent back7 passes', 'Export1 sent back2 passes'])
    await rows[1]?.trigger('click')

    expect(wrapper.emitted('open')).toEqual([['b']])
  })

  it('lists no task when none had a step sent back', async () => {
    vi.mocked(api.fetchProjectAdherence).mockResolvedValue(projectAdherence({ sentBack: 0, mostSentBack: [] }))

    const wrapper = await render()

    expect(wrapper.find('[data-testid="project-adherence-task"]').exists()).toBe(false)
  })

  it('says so for a project whose tasks have no steps', async () => {
    vi.mocked(api.fetchProjectAdherence).mockResolvedValue(
      projectAdherence({ stepsTotal: 0, stepsDone: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0, firstTimeShare: null, deviations: 0, mostSentBack: [] })
    )

    const wrapper = await render()

    expect(wrapper.find('[data-testid="project-adherence-empty"]').exists()).toBe(true)
  })

  it('reads again when the project gains a task', async () => {
    vi.mocked(api.fetchProjectAdherence).mockResolvedValue(projectAdherence())
    const wrapper = await render()

    await wrapper.setProps({ taskCount: 3 })
    await flushPromises()

    expect(api.fetchProjectAdherence).toHaveBeenCalledTimes(2)
  })
})
