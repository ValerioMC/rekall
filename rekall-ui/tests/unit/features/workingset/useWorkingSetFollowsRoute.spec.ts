import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import * as api from '@/features/workingset/api/working-set.api'
import { useWorkingSetFollowsRoute } from '@/features/workingset/composables/useWorkingSetFollowsRoute'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId, TaskId } from '@/model/branded'
import { project, task } from './fixtures'

vi.mock('@/features/workingset/api/working-set.api')

const T1 = 't1' as TaskId
const VEGA = 'vega' as ProjectId

const Page = { render: () => null }
const Follower = defineComponent({
  setup() {
    useWorkingSetFollowsRoute()
    return () => null
  }
})

function routerAt(): Router {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/task/:id/:tab?', name: 'task', component: Page },
      { path: '/projects/:id/:tab?', name: 'project-detail', component: Page },
      { path: '/settings', name: 'settings', component: Page }
    ]
  })
}

async function mountedAt(path: string): Promise<Router> {
  const router = routerAt()
  await router.push(path)
  await router.isReady()
  mount(Follower, { global: { plugins: [router] } })
  await flushPromises()
  return router
}

describe('the working set follows the route', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [], places: [] })
    vi.mocked(api.openTask).mockResolvedValue({ tasks: [], places: [] })
    vi.mocked(api.rememberPlace).mockResolvedValue({ projectId: VEGA, taskId: null })
    const catalog = useConsoleStore()
    catalog.isLoading = false
    catalog.projects = [project('vega', 'Vega', 'c1')]
    catalog.tasks = [task('t1', 'vega')]
  })

  it('opens a task when its page is shown and remembers it as the place in its project', async () => {
    await useWorkingSetStore().load()

    await mountedAt('/task/t1/description')

    expect(api.openTask).toHaveBeenCalledWith(T1)
    expect(api.rememberPlace).toHaveBeenCalledWith(VEGA, T1)
  })

  it('remembers the task list as the place when a project\'s tasks are shown', async () => {
    await useWorkingSetStore().load()

    await mountedAt('/projects/vega')

    expect(api.rememberPlace).toHaveBeenCalledWith(VEGA, null)
    expect(api.openTask).not.toHaveBeenCalled()
  })

  it('does nothing on a project page that is not its task list', async () => {
    await useWorkingSetStore().load()

    await mountedAt('/projects/vega/pages')

    expect(api.rememberPlace).not.toHaveBeenCalled()
  })

  it('does nothing on any other page, and nothing before the set is read', async () => {
    await mountedAt('/task/t1/description')
    await useWorkingSetStore().load()
    await mountedAt('/settings')

    expect(api.openTask).toHaveBeenCalledTimes(1)
  })

  it('does not open a task the catalog does not hold', async () => {
    await useWorkingSetStore().load()

    await mountedAt('/task/unknown/description')

    expect(api.openTask).not.toHaveBeenCalled()
  })
})
