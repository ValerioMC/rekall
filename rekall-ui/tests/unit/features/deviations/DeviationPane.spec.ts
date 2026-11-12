import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import DeviationPane from '@/features/deviations/components/DeviationPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import * as api from '@/features/deviations/api/deviations.api'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskStepId } from '@/model/branded'
import { deviation, taskId } from './fixtures'

vi.mock('@/features/deviations/api/deviations.api', () => ({ fetchDeviations: vi.fn() }))

const task = { id: taskId, title: 'Report builder', anchor: 'project:vega task:report-builder', projectId: 'p1' as ProjectId } as Task
const steps = [
  { id: 's1' as TaskStepId, taskId, title: 'Aggregate', state: 'DONE', position: 0 },
  { id: 's2' as TaskStepId, taskId, title: 'Stream', state: 'OPEN', position: 1 }
] as TaskStep[]

const stubs = { TaskTitleRow: true, AppMarkdownEditor: { props: ['modelValue'], template: '<div data-testid="body">{{ modelValue }}</div>' } }

async function open(onStep: number, deviations = [deviation(1), deviation(2, { stepTitle: null })]) {
  vi.mocked(api.fetchDeviations).mockResolvedValue(deviations)
  const store = useConsoleStore()
  store.tasks = [task]
  store.steps = steps
  store.selectedTaskId = taskId
  store.openDeviation(onStep)
  const wrapper = mount(DeviationPane, { global: { stubs } })
  await flushPromises()
  return { store, wrapper }
}

describe('the deviation pane', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('reads the deviation of the step in view, with the step it names', async () => {
    const { wrapper } = await open(1)

    expect(wrapper.get('[data-testid="deviation-heading"]').text()).toBe('Step 1: Title of step 1')
    expect(wrapper.get('[data-testid="body"]').text()).toBe('What step 1 did differently.')
    expect(wrapper.get('[data-testid="deviation-position"]').text()).toBe('1 of 2')
  })

  it('goes to the next deviation and back, and stops at either end', async () => {
    const { store, wrapper } = await open(1)
    expect(wrapper.get('[data-testid="deviation-previous"]').attributes('disabled')).toBeDefined()

    await wrapper.get('[data-testid="deviation-next"]').trigger('click')

    expect([store.paneFocus, store.selectedDeviationStep]).toEqual(['deviation', 2])
    expect(wrapper.get('[data-testid="deviation-next"]').attributes('disabled')).toBeDefined()

    wrapper.vm.step(-1)

    expect(store.selectedDeviationStep).toBe(1)
  })

  it('jumps to a step from the rail of links', async () => {
    const { store, wrapper } = await open(1)

    await wrapper.findAll('[data-testid="deviation-rail-link"]')[1]!.trigger('click')

    expect(store.selectedDeviationStep).toBe(2)
  })

  it('opens the step the deviation is about, or says the checklist no longer has it', async () => {
    const { store, wrapper } = await open(2)

    await wrapper.get('[data-testid="deviation-open-step"]').trigger('click')

    expect([store.paneFocus, store.stepToOpen]).toEqual(['steps', 's2'])

    store.openDeviation(1)
    store.steps = []
    await flushPromises()

    expect(wrapper.find('[data-testid="deviation-open-step"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('The checklist has no step 1 any more.')
  })

  it('closes back to the notes', async () => {
    const { store, wrapper } = await open(1)

    await wrapper.get('[data-testid="deviation-close"]').trigger('click')

    expect([store.paneFocus, store.selectedDeviationStep]).toEqual(['note', null])
  })

  it('says the work followed the plan only once the task is known to have no deviation', async () => {
    const { wrapper } = await open(3, [])

    expect(wrapper.text()).toContain('The work followed the plan')
  })
})
