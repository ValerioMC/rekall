import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WorkPage from '@/features/work/components/WorkPage.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type * as catalogApi from '@/features/catalog/api/catalog.api'
import type * as stepsApi from '@/features/steps/api/steps.api'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'
import { project, step, task } from './work-fixtures'

const patchStep = vi.fn()
const sendBackStep = vi.fn()
const reviewTask = vi.fn()

vi.mock('@/features/steps/api/steps.api', async (importOriginal) => ({
  ...(await importOriginal<typeof stepsApi>()),
  patchStep: (...args: unknown[]) => patchStep(...args),
  sendBackStep: (...args: unknown[]) => sendBackStep(...args)
}))

vi.mock('@/features/catalog/api/catalog.api', async (importOriginal) => ({
  ...(await importOriginal<typeof catalogApi>()),
  reviewTask: (...args: unknown[]) => reviewTask(...args)
}))

const claimed = (id: string, taskId: string, position: number): TaskStep => ({
  ...step(id, taskId, 'CLAIMED'),
  title: `Claim ${id}`,
  position,
  claimedAt: '2026-11-13T09:00:00Z'
})

function seed(extra: Task[] = [], steps: TaskStep[] = []) {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega')]
  store.tasks = [task('PR-1', { title: 'Plan the launch', stepCount: 5, anchor: 'task:PR-1' }), ...extra]
  store.steps = [claimed('s1', 'PR-1', 1), claimed('s2', 'PR-1', 2), ...steps]
  return { store, work: useWorkStore() }
}

async function mountWork() {
  const wrapper = mount(WorkPage, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

const rowOf = (wrapper: Awaited<ReturnType<typeof mountWork>>, title: string) =>
  wrapper.findAll('[data-testid="work-row"]').find((candidate) => candidate.text().includes(title))!

describe('the row peek', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    patchStep.mockReset()
    sendBackStep.mockReset()
    reviewTask.mockReset()
    document.body.innerHTML = ''
  })

  it('opens under the row with the anchor to copy, and closes on a second press', async () => {
    seed()
    const wrapper = await mountWork()

    await rowOf(wrapper, 'Plan the launch').trigger('click')

    expect(wrapper.find('[data-testid="work-peek"]').exists()).toBe(true)
    expect(wrapper.get('[data-testid="peek-anchor"]').text()).toBe('/rk task:PR-1')
    expect(rowOf(wrapper, 'Plan the launch').attributes('aria-expanded')).toBe('true')

    await rowOf(wrapper, 'Plan the launch').trigger('click')

    expect(wrapper.find('[data-testid="work-peek"]').exists()).toBe(false)
  })

  it('keeps one peek open: opening another row closes the first', async () => {
    seed([task('PR-2', { title: 'Fix the login', reviewActive: true, reviewState: 'CLAIMED' })])
    const wrapper = await mountWork()

    await rowOf(wrapper, 'Plan the launch').trigger('click')
    await rowOf(wrapper, 'Fix the login').trigger('click')

    expect(wrapper.findAll('[data-testid="work-peek"]')).toHaveLength(1)
    expect(wrapper.get('[data-testid="peek-anchor"]').text()).toContain('task:PR-2')
  })

  it('lists up to three claimed steps and counts the rest', async () => {
    const { store } = seed([], [claimed('s3', 'PR-1', 3), claimed('s4', 'PR-1', 4), claimed('s5', 'PR-1', 5)])
    const wrapper = await mountWork()
    expect(store.steps.filter((candidate) => candidate.state === 'CLAIMED')).toHaveLength(5)

    await rowOf(wrapper, 'Plan the launch').trigger('click')

    expect(wrapper.findAll('[data-testid="peek-claim"]')).toHaveLength(3)
    expect(wrapper.get('[data-testid="peek-more"]').text()).toBe('2 more to review')
  })

  it('accepts a claimed step, updates the counts and offers to reopen it', async () => {
    const { store } = seed()
    patchStep.mockImplementation(async (id: TaskStepId) => ({ ...store.steps.find((candidate) => candidate.id === id)!, state: 'DONE', done: true }))
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Plan the launch').trigger('click')

    await wrapper.findAll('[data-testid="peek-accept"]')[0]!.trigger('click')
    await flushPromises()

    expect(patchStep).toHaveBeenCalledWith('s1', { done: true })
    expect(wrapper.get('[data-testid="sub-nav-needs"] [data-testid="sub-nav-count"]').text()).toBe('1')
    const toast = useToastStore().toasts.at(-1)!
    expect(toast.message).toBe('Accepted step 1')
    expect(toast.action?.label).toBe('Reopen')
  })

  it('keeps the accepted step in view with its seal stamped, without buttons', async () => {
    const { store } = seed()
    patchStep.mockImplementation(async (id: TaskStepId) => ({ ...store.steps.find((candidate) => candidate.id === id)!, state: 'DONE', done: true }))
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Plan the launch').trigger('click')

    await wrapper.findAll('[data-testid="peek-accept"]')[0]!.trigger('click')
    await flushPromises()

    const stamped = wrapper.findAll('[data-testid="peek-claim"]')[0]!
    expect(stamped.get('[data-testid="step-seal"]').attributes('data-state')).toBe('DONE')
    expect(stamped.text()).toContain('accepted')
    expect(stamped.find('[data-testid="peek-accept"]').exists()).toBe(false)
  })

  it('sends a claimed step back to its session', async () => {
    const { store } = seed()
    sendBackStep.mockImplementation(async (id: TaskStepId) => ({ ...store.steps.find((candidate) => candidate.id === id)!, state: 'OPEN' }))
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Plan the launch').trigger('click')

    await wrapper.findAll('[data-testid="peek-send-back"]')[1]!.trigger('click')
    await flushPromises()

    expect(sendBackStep).toHaveBeenCalledWith('s2')
    expect(useToastStore().toasts.at(-1)!.message).toBe('Sent step 2 back to the session')
  })

  it('accepts a task handed in whole', async () => {
    const whole = task('PR-2', { title: 'Fix the login', reviewActive: true, reviewState: 'CLAIMED' })
    seed([whole])
    reviewTask.mockImplementation(async (_id: string, reviewState: string) => ({ ...whole, reviewState }))
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Fix the login').trigger('click')

    await wrapper.get('[data-testid="peek-accept"]').trigger('click')
    await flushPromises()

    expect(reviewTask).toHaveBeenCalledWith('PR-2', 'DONE')
    expect(useToastStore().toasts.at(-1)!.action?.label).toBe('Reopen')
  })

  it('sends a task handed in whole back to its session', async () => {
    const whole = task('PR-2', { title: 'Fix the login', reviewActive: true, reviewState: 'CLAIMED' })
    seed([whole])
    reviewTask.mockImplementation(async (_id: string, reviewState: string) => ({ ...whole, reviewState }))
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Fix the login').trigger('click')

    await wrapper.get('[data-testid="peek-send-back"]').trigger('click')
    await flushPromises()

    expect(reviewTask).toHaveBeenCalledWith('PR-2', 'OPEN')
  })

  it('opens the task in the console from its peek', async () => {
    const { store } = seed()
    const push = vi.spyOn(router, 'push').mockResolvedValue(undefined)
    const wrapper = await mountWork()
    await rowOf(wrapper, 'Plan the launch').trigger('click')

    await wrapper.get('[data-testid="peek-open-task"]').trigger('click')

    expect(store.selectedTaskId).toBe('PR-1')
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('shows what runs and what is next when nothing is claimed', async () => {
    const { store } = seed()
    store.steps = [{ ...step('s1', 'PR-1', 'RUNNING'), title: 'Wire the form' }, { ...step('s2', 'PR-1', 'OPEN'), title: 'Add tests' }]
    useWorkStore().view = 'open'
    const wrapper = await mountWork()

    await rowOf(wrapper, 'Plan the launch').trigger('click')

    expect(wrapper.findAll('[data-testid="peek-next"]').map((item) => item.text())).toEqual(['Wire the formworking', 'Add testsnext'])
  })
})
