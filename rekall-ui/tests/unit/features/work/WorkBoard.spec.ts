import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WorkPage from '@/features/work/components/WorkPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { project, step, task } from './work-fixtures'

const stub = { template: '<div />' }

function newRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'work', component: stub },
      { path: '/work/board', name: 'work-board', component: stub },
      { path: '/task/:id/:tab?', name: 'task', component: stub },
      { path: '/console', name: 'console', component: stub },
      { path: '/projects/:id', name: 'project-detail', component: stub }
    ]
  })
}

async function mountBoard() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega')]
  store.tasks = [
    task('PR-1', { title: 'Write the docs', status: 'TODO' }),
    task('PR-2', { title: 'Plan the launch', stepCount: 2 }),
    task('PR-3', { title: 'Fix the login', stepCount: 1 }),
    task('PR-4', { title: 'Ship the build', status: 'DONE' }),
    task('PR-5', { title: 'Ship the docs', status: 'DONE' })
  ]
  store.steps = [step('s1', 'PR-2', 'CLAIMED'), step('s2', 'PR-3', 'RUNNING')]
  const router = newRouter()
  await router.push('/work/board')
  const wrapper = mount(WorkPage, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return { wrapper, router, store, work: useWorkStore() }
}

const cardTitles = (wrapper: Awaited<ReturnType<typeof mountBoard>>['wrapper'], lane: string): string[] =>
  wrapper.findAll(`[data-testid="board-lane-${lane}"] [data-testid="board-card-title"]`).map((title) => title.text())

describe('the board', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    document.body.innerHTML = ''
  })

  it('deals each task into the lane of its state', async () => {
    const { wrapper } = await mountBoard()

    expect(cardTitles(wrapper, 'todo')).toEqual(['Write the docs'])
    expect(cardTitles(wrapper, 'progress')).toEqual(['Fix the login'])
    expect(cardTitles(wrapper, 'review')).toEqual(['Plan the launch'])
  })

  it('counts the same tasks the ledger counts on Open, and files the rest on a shelf', async () => {
    const { wrapper, work } = await mountBoard()

    const live = wrapper
      .findAll('[data-testid="board-lane-count"]')
      .map((count) => Number(count.text()))
      .slice(0, 3)
      .reduce((sum, count) => sum + count, 0)

    expect(live).toBe(work.counts.open)
    expect(wrapper.get('[data-testid="board-filed-count"]').text()).toBe(String(work.counts.filed))
  })

  it('narrows the lanes to what the filter matches', async () => {
    const { wrapper } = await mountBoard()

    await wrapper.get('[data-testid="work-filter"]').setValue('login')

    expect(cardTitles(wrapper, 'progress')).toEqual(['Fix the login'])
    expect(cardTitles(wrapper, 'todo')).toEqual([])
  })

  it('opens a card in a sheet with the peek of the task', async () => {
    const { wrapper } = await mountBoard()

    await wrapper.get('[data-testid="board-lane-review"] [data-testid="board-card"]').trigger('click')

    expect(wrapper.get('[data-testid="task-sheet"]').text()).toContain('Plan the launch')
    expect(wrapper.get('[data-testid="task-sheet"] [data-testid="work-peek"]').text()).toContain('Step s1')
  })

  it('closes the sheet on Escape', async () => {
    const { wrapper } = await mountBoard()
    await wrapper.get('[data-testid="board-card"]').trigger('click')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()

    expect(wrapper.find('[data-testid="task-sheet"]').exists()).toBe(false)
  })

  it('closes the sheet on a press outside it', async () => {
    const { wrapper } = await mountBoard()
    await wrapper.get('[data-testid="board-card"]').trigger('click')

    await wrapper.get('[data-testid="sheet-scrim"]').trigger('click')

    expect(wrapper.find('[data-testid="task-sheet"]').exists()).toBe(false)
  })

  it('opens the task as a page from its sheet', async () => {
    const { wrapper, router } = await mountBoard()
    const push = vi.spyOn(router, 'push')
    await wrapper.get('[data-testid="board-lane-review"] [data-testid="board-card"]').trigger('click')

    await wrapper.get('[data-testid="sheet-open-page"]').trigger('click')

    expect(push).toHaveBeenCalledWith({ name: 'task', params: { id: 'PR-2', tab: 'plan' } })
  })

  it('opens the ledger on Filed from the shelf', async () => {
    const { wrapper, router, work } = await mountBoard()

    await wrapper.get('[data-testid="board-open-shelf"]').trigger('click')
    await flushPromises()

    expect(work.view).toBe('filed')
    expect(router.currentRoute.value.name).toBe('work')
  })

  it('switches between the ledger and the board with the layout toggle', async () => {
    const { wrapper, router } = await mountBoard()

    await wrapper.get('[data-testid="segment-ledger"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('work')
    expect(wrapper.find('[data-testid="work-ledger"]').exists()).toBe(true)

    await wrapper.get('[data-testid="segment-board"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('work-board')
    expect(wrapper.find('[data-testid="work-board"]').exists()).toBe(true)
  })
})
