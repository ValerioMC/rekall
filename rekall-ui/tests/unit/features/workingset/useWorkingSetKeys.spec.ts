import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import * as api from '@/features/workingset/api/working-set.api'
import { useWorkingSetKeys } from '@/features/workingset/composables/useWorkingSetKeys'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useModalGate } from '@/composables/useModalGate'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId, TaskId } from '@/model/branded'
import { company, project, task } from './fixtures'

vi.mock('@/features/workingset/api/working-set.api')

const VEGA = 'vega' as ProjectId
const ATLAS = 'atlas' as ProjectId

const Page = { render: () => null }
const Keys = defineComponent({
  setup() {
    useWorkingSetKeys()
    return () => null
  }
})

let mounted: VueWrapper | null = null

async function keysAt(path: string): Promise<Router> {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/task/:id/:tab?', name: 'task', component: Page },
      { path: '/projects/:id/:tab?', name: 'project-detail', component: Page }
    ]
  })
  await router.push(path)
  await router.isReady()
  mounted = mount(Keys, { global: { plugins: [router] } })
  return router
}

async function press(key: string, init: KeyboardEventInit = {}, target: EventTarget = window): Promise<void> {
  target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...init }))
  await flushPromises()
}

describe('the working-set keys', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    const catalog = useConsoleStore()
    catalog.companies = [company('c1', 'Northwind')]
    catalog.projects = [project('vega', 'Vega', 'c1'), project('atlas', 'Atlas', 'c1')]
    catalog.tasks = [task('t1', 'vega'), task('t2', 'vega'), task('t3', 'atlas')]
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({
      tasks: [{ taskId: 't1' as TaskId, projectId: VEGA }, { taskId: 't2' as TaskId, projectId: VEGA }, { taskId: 't3' as TaskId, projectId: ATLAS }],
      places: [{ projectId: VEGA, taskId: 't2' as TaskId }]
    })
    await useWorkingSetStore().load()
  })

  afterEach(() => {
    mounted?.unmount()
    mounted = null
  })

  it('opens the n-th project of the bar on its number, at the place last seen there', async () => {
    const router = await keysAt('/task/t3/description')

    await press('2')

    expect(router.currentRoute.value.params.id).toBe('t2')
  })

  it('does nothing on a number with no project', async () => {
    const router = await keysAt('/task/t3/description')

    await press('7')

    expect(router.currentRoute.value.params.id).toBe('t3')
  })

  it('steps through the open tasks across projects and wraps', async () => {
    const router = await keysAt('/task/t2/description')

    await press(']')
    expect(router.currentRoute.value.params.id).toBe('t3')
    await press(']')
    expect(router.currentRoute.value.params.id).toBe('t1')
    await press('[')
    expect(router.currentRoute.value.params.id).toBe('t3')
  })

  it('flips back to the previous place, and to its task list once that task is closed', async () => {
    const store = useWorkingSetStore()
    const router = await keysAt('/task/t1/description')
    store.noteVisit({ projectId: VEGA, taskId: 't1' as TaskId })
    store.noteVisit({ projectId: ATLAS, taskId: 't3' as TaskId })

    await press('`')
    expect(router.currentRoute.value.params.id).toBe('t1')

    store.noteVisit({ projectId: ATLAS, taskId: 't3' as TaskId })
    store.noteVisit({ projectId: VEGA, taskId: 't1' as TaskId })
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: ATLAS, taskId: null })
    await store.close('t3' as TaskId)
    await press('`')
    expect(router.currentRoute.value.fullPath).toBe('/projects/atlas')
  })

  it('says nothing when there is no previous place', async () => {
    const router = await keysAt('/task/t1/description')

    await press('`')

    expect(router.currentRoute.value.params.id).toBe('t1')
  })

  it('is ignored while typing, while a dialog is open and with a modifier held', async () => {
    const router = await keysAt('/task/t3/description')
    const field = document.createElement('input')
    document.body.appendChild(field)

    await press('1', {}, field)
    await press('1', { ctrlKey: true })
    await press('1', { metaKey: true })
    await press('1', { altKey: true })
    useModalGate().open()
    await press('1')
    useModalGate().close()
    field.remove()

    expect(router.currentRoute.value.params.id).toBe('t3')
  })
})
