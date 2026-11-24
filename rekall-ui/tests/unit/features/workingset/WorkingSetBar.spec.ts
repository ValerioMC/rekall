import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import * as api from '@/features/workingset/api/working-set.api'
import WorkingSetBar from '@/features/workingset/components/WorkingSetBar.vue'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId, TaskId } from '@/model/branded'
import { company, project, task } from './fixtures'

vi.mock('@/features/workingset/api/working-set.api')

const T1 = 't1' as TaskId
const T2 = 't2' as TaskId
const T3 = 't3' as TaskId
const VEGA = 'vega' as ProjectId
const ATLAS = 'atlas' as ProjectId

const Page = { render: () => null }

async function barAt(path: string): Promise<{ bar: ReturnType<typeof mount>; router: Router }> {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/task/:id/:tab?', name: 'task', component: Page },
      { path: '/projects/:id/:tab?', name: 'project-detail', component: Page },
      { path: '/settings', name: 'settings', component: Page }
    ]
  })
  await router.push(path)
  await router.isReady()
  const bar = mount(WorkingSetBar, { global: { plugins: [router] } })
  await flushPromises()
  return { bar, router }
}

describe('the working-set bar', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    const catalog = useConsoleStore()
    catalog.companies = [company('c1', 'Northwind')]
    catalog.projects = [project('vega', 'Vega', 'c1'), project('atlas', 'Atlas', 'c1')]
    catalog.tasks = [task('t1', 'vega'), task('t2', 'vega'), task('t3', 'atlas')]
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({
      tasks: [{ taskId: T1, projectId: VEGA }, { taskId: T2, projectId: VEGA }, { taskId: T3, projectId: ATLAS }],
      places: [{ projectId: ATLAS, taskId: T3 }]
    })
    await useWorkingSetStore().load()
  })

  it('shows both rows on a task page, marking its project and task', async () => {
    const { bar } = await barAt('/task/t2/description')

    expect(bar.findAll('[data-testid="working-set-project"]')).toHaveLength(2)
    expect(bar.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('aria-selected'))).toEqual(['false', 'true'])
  })

  it('shows the task list as the place on a project page', async () => {
    const { bar } = await barAt('/projects/vega')

    expect(bar.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('true')
  })

  it('is hidden on other pages and while nothing is open', async () => {
    const { bar: settings } = await barAt('/settings')
    expect(settings.find('[data-testid="working-set-bar"]').exists()).toBe(false)

    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [], places: [] })
    await useWorkingSetStore().load()
    const { bar: empty } = await barAt('/task/t1/description')
    expect(empty.find('[data-testid="working-set-bar"]').exists()).toBe(false)
  })

  it('goes to the place last seen in a project when its tab is pressed', async () => {
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-project"]')[0]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t3')
  })

  it('goes to the task list of a project whose remembered task is no longer open', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: VEGA }, { taskId: T3, projectId: ATLAS }], places: [{ projectId: VEGA, taskId: T2 }] })
    await useWorkingSetStore().load()
    const { bar, router } = await barAt('/task/t3/description')

    await bar.findAll('[data-testid="working-set-project"]')[1]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/projects/vega')
  })

  it('follows where the server says the project lands when the task in view is closed', async () => {
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T2 })
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-task-close"]')[0]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t2')
  })

  it('closes a task that is not in view without leaving the page', async () => {
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T1 })
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-task-close"]')[1]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t1')
    expect(useWorkingSetStore().isOpen(T2)).toBe(false)
  })
})
