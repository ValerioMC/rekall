import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WorkPage from '@/features/work/components/WorkPage.vue'
import { router } from '@/router'
import { reviewQueue } from '@/features/catalog/model/review'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { ProjectId } from '@/model/branded'
import { project, step, task } from './work-fixtures'

const orion = 'p2' as ProjectId

function seed() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega'), project('p2', 'Orion')]
  store.tasks = [
    task('PR-1', { title: 'Plan the launch', stepCount: 2 }),
    task('PR-2', { title: 'Write the docs', status: 'TODO' }),
    task('PR-3', { title: 'Ship the build', status: 'DONE' }),
    task('PR-4', { title: 'Orion kickoff', projectId: orion, projectTitle: 'Orion', status: 'DONE' }),
    task('PR-5', { title: 'Fix the login', reviewActive: true, reviewState: 'CLAIMED' })
  ]
  store.steps = [step('s1', 'PR-1', 'CLAIMED'), step('s2', 'PR-1', 'CLAIMED'), step('s3', 'PR-1', 'RUNNING')]
  return { store, work: useWorkStore() }
}

async function mountWork() {
  const wrapper = mount(WorkPage, { global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

const titles = (wrapper: Awaited<ReturnType<typeof mountWork>>): string[] =>
  wrapper.findAll('[data-testid="work-row-title"]').map((title) => title.text())

describe('WorkPage', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
  })

  it('opens on Needs me, listing only the tasks with something to review', async () => {
    seed()

    const wrapper = await mountWork()

    expect(titles(wrapper).sort()).toEqual(['Fix the login', 'Plan the launch'])
  })

  it('counts on the Needs me tab what the review queue counts', async () => {
    const { store } = seed()

    const wrapper = await mountWork()

    const count = wrapper.get('[data-testid="sub-nav-needs"] [data-testid="sub-nav-count"]').text()
    expect(count).toBe(String(reviewQueue(store.tasks, store.steps).length))
  })

  it('lists every task that is not filed under its project on Open, the finished ones folded', async () => {
    const { work } = seed()
    work.view = 'open'

    const wrapper = await mountWork()

    expect(wrapper.findAll('[data-testid="work-group"]').map((group) => group.find('a').text())).toEqual(['Vega'])
    expect(titles(wrapper)).toHaveLength(3)
    expect(wrapper.get('[data-testid="work-filed-fold"]').text()).toBe('1 filed')
  })

  it('unfolds the filed line of a project into its finished tasks', async () => {
    const { work } = seed()
    work.view = 'open'
    const wrapper = await mountWork()

    await wrapper.get('[data-testid="work-filed-fold"]').trigger('click')

    expect(titles(wrapper)).toContain('Ship the build')
  })

  it('lists the finished tasks of every project on Filed, with no fold', async () => {
    const { work } = seed()
    work.view = 'filed'

    const wrapper = await mountWork()

    expect(titles(wrapper).sort()).toEqual(['Orion kickoff', 'Ship the build'])
    expect(wrapper.find('[data-testid="work-filed-fold"]').exists()).toBe(false)
  })

  it('lists the tasks a session works on under Running', async () => {
    const { work } = seed()
    work.view = 'running'

    const wrapper = await mountWork()

    expect(titles(wrapper)).toEqual(['Plan the launch'])
  })

  it('says nothing waits when no step is claimed', async () => {
    const { store } = seed()
    store.steps = []
    store.tasks = store.tasks.map((candidate) => ({ ...candidate, reviewActive: false }))

    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-nothing-waits"]').text()).toContain('Nothing waits for you.')
  })

  it('narrows the rows to the titles and codes that match the filter', async () => {
    const { work } = seed()
    work.view = 'open'
    const wrapper = await mountWork()

    await wrapper.get('[data-testid="work-filter"]').setValue('docs')

    expect(titles(wrapper)).toEqual(['Write the docs'])
  })

  it('says no task matches when the filter finds none', async () => {
    const { work } = seed()
    work.view = 'open'
    const wrapper = await mountWork()

    await wrapper.get('[data-testid="work-filter"]').setValue('zzz')

    expect(wrapper.find('[data-testid="work-no-match"]').exists()).toBe(true)
  })

  it('sorts a project by what needs you when the Needs you key is pressed', async () => {
    const { work } = seed()
    work.view = 'open'
    const wrapper = await mountWork()

    await wrapper.findAll('[role="row"] button').find((key) => key.text() === 'Needs you')!.trigger('click')

    expect(work.sort).toBe('needs')
    expect(titles(wrapper)[0]).toBe('Plan the launch')
  })

  it('draws the rows tighter in the compact density', async () => {
    const { work } = seed()
    work.density = 'compact'

    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-row"]').classes()).toContain('h-[34px]')
  })

  it('opens the task a row names in the console', async () => {
    const { store } = seed()
    const push = vi.spyOn(router, 'push').mockResolvedValue(undefined)
    const wrapper = await mountWork()

    await wrapper.findAll('[data-testid="work-row"]').find((row) => row.text().includes('Plan the launch'))!.trigger('click')
    await flushPromises()

    expect(store.selectedTaskId).toBe('PR-1')
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('shows the project with its company wherever tasks are grouped', async () => {
    const { work } = seed()
    work.view = 'open'

    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-group"]').text()).toContain('Acme')
  })

  it('asks for a new task through its button', async () => {
    seed()
    const wrapper = await mountWork()

    await wrapper.get('[data-testid="new-task"]').trigger('click')

    expect(wrapper.emitted('newTask')).toHaveLength(1)
  })
})
