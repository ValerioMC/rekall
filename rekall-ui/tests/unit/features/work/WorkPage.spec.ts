import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WorkPage from '@/features/work/components/WorkPage.vue'
import { router } from '@/router'
import { reviewQueue } from '@/features/catalog/model/review'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { ProjectId, TaskId } from '@/model/branded'
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
  wrapper
    .findAll('[data-testid="work-row"]')
    .map((row) => `[${row.get('[data-testid="work-row-code"]').text()}] ${row.get('[data-testid="work-row-title"]').text()}`)

describe('WorkPage', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
  })

  it('opens on Needs me, listing only the tasks with something to review', async () => {
    seed()

    const wrapper = await mountWork()

    expect(titles(wrapper).sort()).toEqual(['[PR-1] Plan the launch', '[PR-5] Fix the login'])
  })

  it('shows a task code in a column of its own, apart from the title', async () => {
    seed()

    const wrapper = await mountWork()
    const row = wrapper.findAll('[data-testid="work-row"]').find((candidate) => candidate.text().includes('Plan the launch'))!

    expect(row.get('[data-testid="work-row-code"]').text()).toBe('PR-1')
    expect(row.get('[data-testid="work-row-title"]').text()).toBe('Plan the launch')
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

    expect(titles(wrapper)).toContain('[PR-3] Ship the build')
  })

  it('lists the finished tasks of every project on Filed, with no fold', async () => {
    const { work } = seed()
    work.view = 'filed'

    const wrapper = await mountWork()

    expect(titles(wrapper).sort()).toEqual(['[PR-3] Ship the build', '[PR-4] Orion kickoff'])
    expect(wrapper.find('[data-testid="work-filed-fold"]').exists()).toBe(false)
  })

  it('lists the tasks a session works on under Running', async () => {
    const { work } = seed()
    work.view = 'running'

    const wrapper = await mountWork()

    expect(titles(wrapper)).toEqual(['[PR-1] Plan the launch'])
  })

  it('lists the tasks of an epic under it, and folds them from the epic row', async () => {
    const { store, work } = seed()
    store.tasks = [
      task('PR-6', { title: 'Checkout rework', kind: 'EPIC' }),
      task('PR-7', { title: 'Pay by card', epicId: 'PR-6' as TaskId }),
      task('PR-8', { title: 'Pay by wire', epicId: 'PR-6' as TaskId }),
      task('PR-9', { title: 'Unrelated' })
    ]
    work.view = 'open'
    const wrapper = await mountWork()

    expect(titles(wrapper)).toEqual(['[PR-9] Unrelated', '[PR-6] Checkout rework', '[PR-8] Pay by wire', '[PR-7] Pay by card'])
    expect(wrapper.get('[data-testid="work-epic-summary"]').text()).toBe('2 tasks')

    await wrapper.get('[data-testid="work-epic-toggle"]').trigger('click')

    expect(titles(wrapper)).toEqual(['[PR-9] Unrelated', '[PR-6] Checkout rework'])
  })

  it('names the epic of a task listed without it', async () => {
    const { store, work } = seed()
    store.tasks = [task('PR-6', { title: 'Checkout rework', kind: 'EPIC', status: 'DONE' }), task('PR-7', { title: 'Pay by card', epicId: 'PR-6' as TaskId })]
    work.view = 'open'
    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-row-epic"]').text()).toBe('Checkout rework')
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

    expect(titles(wrapper)).toEqual(['[PR-2] Write the docs'])
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
    expect(titles(wrapper)[0]).toBe('[PR-1] Plan the launch')
  })

  it('draws the rows tighter in the compact density', async () => {
    const { work } = seed()
    work.density = 'compact'

    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-row"]').classes()).toContain('h-[34px]')
  })

  it('shows the project with its company wherever tasks are grouped', async () => {
    const { work } = seed()
    work.view = 'open'

    const wrapper = await mountWork()

    expect(wrapper.get('[data-testid="work-group"]').text()).toContain('Acme')
  })
})

describe('WorkPage composer', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    document.body.innerHTML = ''
  })

  it('opens the new task dialog on the picked project with the typed title through Plan a task', async () => {
    const { store } = seed()
    store.projects = store.projects.map((candidate) => ({ ...candidate, repoFolder: '/code' }))
    const wrapper = mount(WorkPage, { attachTo: document.body, global: { plugins: [router] } })
    await flushPromises()

    await wrapper.get('[data-testid="quick-task-input"]').setValue('Ship the export')
    await wrapper.get('[data-testid="split-button-arrow"]').trigger('click')
    document.body.querySelector<HTMLElement>('[data-testid="split-button-option-plan"]')!.click()

    expect(useTaskEditor().editing.value).toMatchObject({ projectId: 'p1', title: 'Ship the export' })
    useTaskEditor().close()
  })

  it('narrows the composer and the list to the project in view and drops the project headers', async () => {
    const { store } = seed()
    store.setScope(store.projects[0]!.companyId, 'p1' as ProjectId)
    useWorkStore().view = 'open'
    const wrapper = mount(WorkPage, { attachTo: document.body, global: { plugins: [router] } })
    await flushPromises()

    expect(wrapper.find('[data-testid="composer-project"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="work-group"] header').exists()).toBe(false)
    expect(wrapper.find('[data-testid="project-adherence"]').exists()).toBe(true)
  })
})

describe('WorkPage pages', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
  })

  function seedMany(count: number) {
    const { store, work } = seed()
    store.tasks = Array.from({ length: count }, (_, index) => task(`PR-${index + 1}`, { title: `Task number ${index + 1}`, updatedAt: `2026-09-${String(10 + (index % 20)).padStart(2, '0')}T10:00:00Z` }))
    store.steps = []
    work.view = 'open'
    return work
  }

  it('lists one page of the tasks and says which ones out of how many', async () => {
    seedMany(60)

    const wrapper = await mountWork()

    expect(titles(wrapper)).toHaveLength(25)
    expect(wrapper.get('[data-testid="work-pager-range"]').text()).toBe('1–25 of 60 tasks')
    expect(wrapper.get('[data-testid="work-pager-previous"]').attributes('disabled')).toBeDefined()
  })

  it('moves to the next page and back', async () => {
    const work = seedMany(60)
    const wrapper = await mountWork()
    const firstPage = titles(wrapper)

    await wrapper.get('[data-testid="work-pager-next"]').trigger('click')

    expect(wrapper.get('[data-testid="work-pager-range"]').text()).toBe('26–50 of 60 tasks')
    expect(titles(wrapper).some((title) => firstPage.includes(title))).toBe(false)
    expect(work.paging.page).toBe(2)

    await wrapper.get('[data-testid="work-pager-previous"]').trigger('click')

    expect(titles(wrapper)).toEqual(firstPage)
  })

  it('goes back to the first page when the filter changes', async () => {
    seedMany(60)
    const wrapper = await mountWork()
    await wrapper.get('[data-testid="work-pager-next"]').trigger('click')

    await wrapper.get('[data-testid="work-filter"]').setValue('Task number 1')
    await flushPromises()

    expect(wrapper.find('[data-testid="work-pager"]').exists()).toBe(false)
    expect(titles(wrapper).length).toBeGreaterThan(0)
  })

  it('shows no pager when everything fits on one page', async () => {
    seedMany(10)

    const wrapper = await mountWork()

    expect(wrapper.find('[data-testid="work-pager"]').exists()).toBe(false)
  })
})
