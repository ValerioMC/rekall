import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import CommandPalette from '@/features/search/components/CommandPalette.vue'
import { usePaletteStore } from '@/features/search/store/palette.store'
import { useConsoleStore } from '@/stores/console.store'
import type { SearchHit } from '@/features/search/model/search'
import type { TaskId, TaskStepId } from '@/model/branded'
import { project, task } from '../work/work-fixtures'

const searchText = vi.fn<(term: string) => Promise<SearchHit[]>>()
vi.mock('@/features/search/api/search.api', () => ({ searchText: (term: string) => searchText(term) }))

const Blank = { render: () => null }

const stepHit: SearchHit = {
  kind: 'STEP',
  taskId: 't1' as TaskId,
  stepId: 's1' as TaskStepId,
  documentId: null,
  title: 'Wire the batch',
  where: 'project:vega task:t1',
  excerpt: 'Schedule the settlement batch at 02:00.'
}

function routerWithTaskPage(): Router {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: Blank },
      { path: '/task/:id/:tab?/:noteId?', name: 'task', component: Blank },
      { path: '/:rest(.*)', component: Blank }
    ]
  })
}

async function mountPalette() {
  const router = routerWithTaskPage()
  await router.push('/')
  const wrapper = mount(CommandPalette, { global: { plugins: [router] }, attachTo: document.body })
  return { wrapper, router }
}

function seedStore(): ReturnType<typeof useConsoleStore> {
  const store = useConsoleStore()
  store.projects = [project('vega', 'Vega Platform')]
  store.tasks = [task('t1', { title: 'Report builder' }), task('t2', { title: 'Retry policy' })]
  return store
}

describe('the command palette', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    searchText.mockReset().mockResolvedValue([stepHit])
  })

  afterEach(() => {
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  it('lists the screens, projects and tasks before anything is typed', async () => {
    seedStore()
    const { wrapper } = await mountPalette()

    const groups = wrapper.findAll('[role="group"]').map((group) => group.attributes('aria-label'))

    expect(groups).toEqual(['Go to', 'Projects', 'Tasks'])
    wrapper.unmount()
  })

  it('opens the task chosen with the arrows on its page, and closes', async () => {
    seedStore()
    usePaletteStore().show()
    const { wrapper, router } = await mountPalette()
    const input = wrapper.get('[data-testid="palette-input"]')

    await input.setValue('retry')
    await input.trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/t2')
    expect(usePaletteStore().isOpen).toBe(false)
    wrapper.unmount()
  })

  it('walks the entries with the arrow keys and wraps at the ends', async () => {
    seedStore()
    const { wrapper } = await mountPalette()
    const input = wrapper.get('[data-testid="palette-input"]')
    await input.setValue('report')

    expect(wrapper.findAll('[role="option"]')).toHaveLength(2)
    await input.trigger('keydown', { key: 'ArrowUp' })

    const selected = wrapper.findAll('[role="option"]').filter((option) => option.attributes('aria-selected') === 'true')
    expect(selected).toHaveLength(1)
    expect(selected[0]!.text()).toContain('Report builder')
    wrapper.unmount()
  })

  it('searches a phrase once typing pauses, and opens a hit on the tab that shows it', async () => {
    const store = seedStore()
    store.openSearchHit = vi.fn()
    const { wrapper, router } = await mountPalette()
    const input = wrapper.get('[data-testid="palette-input"]')

    await input.setValue('settlement batch')
    expect(searchText).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(250)
    await flushPromises()
    expect(searchText).toHaveBeenCalledWith('settlement batch')

    await wrapper.findAll('[data-testid="palette-entry"]').find((entry) => entry.text().includes('Wire the batch'))!.trigger('mousedown')
    await flushPromises()

    expect(store.openSearchHit).toHaveBeenCalledWith(stepHit)
    expect(router.currentRoute.value.fullPath).toBe('/task/t1/plan')
    wrapper.unmount()
  })

  it('never searches the text for an anchor, which the titles already resolve', async () => {
    seedStore()
    const { wrapper } = await mountPalette()

    await wrapper.get('[data-testid="palette-input"]').setValue('project:vega task:t1')
    await vi.advanceTimersByTimeAsync(250)

    expect(searchText).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('says so when nothing matches', async () => {
    seedStore()
    const { wrapper } = await mountPalette()

    await wrapper.get('[data-testid="palette-input"]').setValue('zz')

    expect(wrapper.get('[data-testid="palette-empty"]').text()).toBe('Nothing matches.')
    wrapper.unmount()
  })
})
