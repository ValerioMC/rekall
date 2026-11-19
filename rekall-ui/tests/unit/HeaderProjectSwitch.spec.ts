import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import HeaderProjectSwitch from '@/components/shell/HeaderProjectSwitch.vue'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { useConsoleStore } from '@/stores/console.store'
import type { CompanyId, TaskId } from '@/model/branded'
import { installStorage } from './support/memoryStorage'
import { project, task } from './features/work/work-fixtures'

const Blank = { render: () => null }

let pinia: Pinia
let router: Router
let wrapper: VueWrapper | null = null

async function mountAt(path: string): Promise<VueWrapper> {
  router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'work', component: Blank },
      { path: '/task/:id/:tab?', name: 'task', component: Blank },
      { path: '/projects/:id/:tab?', name: 'project-detail', component: Blank }
    ]
  })
  await router.push(path)
  const store = useConsoleStore()
  store.companies = [{ id: 'c1' as CompanyId, name: 'Acme', description: null, logoImageVersion: null, projectCount: 2, taskCount: 1, updatedAt: '' }]
  store.projects = [project('p1', 'Vega'), project('p2', 'Atlas')]
  store.tasks = [task('t1', { projectId: 'p2' as never })]
  wrapper = mount(HeaderProjectSwitch, { global: { plugins: [pinia, router] }, attachTo: document.body })
  return wrapper
}

async function choose(view: VueWrapper, title: string): Promise<void> {
  await view.get('[data-testid="scope-trigger"]').trigger('click')
  await flushPromises()
  const row = view.findAll('[data-testid="scope-project"]').find((entry) => entry.text().includes(title))!
  await row.trigger('click')
  await flushPromises()
}

beforeEach(() => {
  document.body.innerHTML = ''
  installStorage()
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => wrapper?.unmount())

describe('the header project switch', () => {
  it('reads Projects when no project is in view', async () => {
    const view = await mountAt('/')

    expect(view.get('[data-testid="scope-trigger-title"]').text()).toBe('Projects')
  })

  it("names the project of the open task and remembers it as recent", async () => {
    useConsoleStore().selectedTaskId = 't1' as TaskId
    const view = await mountAt('/task/t1')

    expect(view.get('[data-testid="scope-trigger-title"]').text()).toBe('Atlas')
    expect(useRecentProjectsStore().ids).toEqual(['p2'])
  })

  it('narrows Work to the chosen project from any other screen', async () => {
    const view = await mountAt('/task/t1')

    await choose(view, 'Vega')

    expect(useConsoleStore().scopeProject).toBe('p1')
    expect(router.currentRoute.value.fullPath).toBe('/')
  })

  it('keeps a project page on its tab when the project changes', async () => {
    const view = await mountAt('/projects/p1/pages')
    useWorkStore().focusProject('p1' as never)

    await choose(view, 'Atlas')

    expect(router.currentRoute.value.fullPath).toBe('/projects/p2/pages')
  })
})
