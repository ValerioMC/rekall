import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import CommandPalette from '@/features/search/components/CommandPalette.vue'
import { usePaletteStore } from '@/features/search/store/palette.store'
import { usePinnedProjectsStore } from '@/features/catalog/store/pinnedProjects.store'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useConsoleStore } from '@/stores/console.store'
import type { SearchHit } from '@/features/search/model/search'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'
import { project, step, task } from '../work/work-fixtures'

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

  it('lists the projects, tasks and screens before anything is typed, when nothing yet leads', async () => {
    seedStore()
    const { wrapper } = await mountPalette()

    const groups = wrapper.findAll('[role="group"]').map((group) => group.attributes('aria-label'))

    expect(groups).toEqual(['Projects', 'Tasks', 'Go to'])
    wrapper.unmount()
  })

  it("opens on the reader's pinned and recent projects, and on the tasks waiting on a review", async () => {
    const store = seedStore()
    store.projects = [project('vega', 'Vega Platform'), project('orion', 'Orion')]
    usePinnedProjectsStore().toggle('orion' as ProjectId)
    useRecentProjectsStore().remember('vega' as ProjectId)
    store.steps = [step('s9', 't2', 'CLAIMED')]
    const { wrapper } = await mountPalette()

    const sections = wrapper.findAll('[role="group"]').map((group) => group.attributes('aria-label'))
    const projects = wrapper.findAll('[role="group"]')[0]!.findAll('[data-testid="palette-title"]').map((title) => title.text())

    expect(sections).toEqual(['Your projects', 'Waiting on you'])
    expect(projects).toEqual(['Orion', 'Vega Platform'])
    wrapper.unmount()
  })

  it('counts each group in a row of tabs and narrows the list to the one pressed', async () => {
    seedStore()
    const { wrapper } = await mountPalette()
    await wrapper.get('[data-testid="palette-input"]').setValue('/rk project:vega')

    const tabs = wrapper.findAll('[data-testid="palette-scope"]')
    expect(tabs.map((tab) => tab.text())).toEqual(['All', 'Projects 1', 'Tasks 2', 'Notes 0', 'Go to 0', 'In the text'])

    await tabs[2]!.trigger('mousedown')

    expect(wrapper.findAll('[role="group"]').map((group) => group.attributes('aria-label'))).toEqual(['Tasks'])
    expect(tabs[2]!.attributes('aria-selected')).toBe('true')
    wrapper.unmount()
  })

  it('walks the tabs with Tab and Shift+Tab, and starts the list over on the new one', async () => {
    seedStore()
    const { wrapper } = await mountPalette()
    const input = wrapper.get('[data-testid="palette-input"]')
    await input.setValue('/rk project:vega')

    await input.trigger('keydown', { key: 'Tab' })
    expect(wrapper.findAll('[role="group"]').map((group) => group.attributes('aria-label'))).toEqual(['Projects'])

    await input.trigger('keydown', { key: 'Tab', shiftKey: true })
    await input.trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(wrapper.get('[aria-selected="true"][role="tab"]').text()).toContain('In the text')
    wrapper.unmount()
  })

  it('offers every row of a group that shows only a few, and opens that group on its key', async () => {
    const store = seedStore()
    store.tasks = Array.from({ length: 6 }, (_, index) => task(`m${index}`, { title: `Batch job ${index}` }))
    const { wrapper } = await mountPalette()
    await wrapper.get('[data-testid="palette-input"]').setValue('/rk batch')

    expect(wrapper.findAll('[data-testid="palette-entry"]')).toHaveLength(4)
    await wrapper.get('[data-testid="palette-widen"]').trigger('mousedown')

    expect(wrapper.findAll('[data-testid="palette-entry"]')).toHaveLength(6)
    expect(wrapper.find('[data-testid="palette-widen"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows where a task is and what is waiting on it', async () => {
    seedStore()
    const { wrapper } = await mountPalette()

    await wrapper.get('[data-testid="palette-input"]').setValue('/rk retry')
    const row = wrapper.get('[data-testid="palette-entry"]')

    expect(row.get('[data-testid="palette-anchor"]').text()).toBe('project:vega task:t2'.split(' ')[1])
    expect(row.text()).toContain('vega')
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
    expect(selected[0]!.text()).toContain('Report')
    expect(selected[0]!.text()).toContain('Time tracked')
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
