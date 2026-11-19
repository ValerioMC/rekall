import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectComponentsPage from '@/features/project/components/ProjectComponentsPage.vue'
import * as api from '@/features/project/api/project-components.api'
import { openTerminal } from '@/features/claude/api/terminal.api'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

vi.mock('@/features/project/api/project-components.api')
vi.mock('@/features/claude/api/terminal.api', () => ({
  openTerminal: vi.fn(async (target: unknown) => ({ id: 't-1', kind: 'TERMINAL', target })),
  fetchTerminals: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

const PROJECT = 'p-1' as ProjectId

const DESCRIPTION = '## Purpose\nIssues invoices.\n\n## Interfaces\n| Route | Does |\n|---|---|\n| POST /invoices | Creates one |'

function component(id: string, position: number, name = id): ProjectComponent {
  return {
    id: id as ProjectComponentId,
    projectId: PROJECT,
    name,
    folder: `/work/${id}`,
    summary: `About ${id}`,
    description: DESCRIPTION,
    position,
    createdAt: '2026-10-08T10:00:00Z',
    updatedAt: '2026-10-08T10:00:00Z'
  }
}

let pinia: Pinia
let wrapper: VueWrapper | null = null

async function render(repoFolder: string | null = '/work', selectedId: string | null = null): Promise<VueWrapper> {
  await useProjectComponentsStore().load(PROJECT)
  const project = { id: PROJECT, title: 'Vega', repoFolder } as unknown as Project
  wrapper = mount(ProjectComponentsPage, {
    props: { project, selectedId: selectedId as ProjectComponentId | null },
    global: { plugins: [pinia], stubs: { AppMarkdownEditor: { props: ['modelValue', 'readonly'], template: '<div data-testid="markdown" :data-readonly="readonly">{{ modelValue }}</div>' } } }
  })
  await flushPromises()
  return wrapper
}

const names = (page: VueWrapper): string[] => page.findAll('[data-testid="component-row-name"]').map((name) => name.text())

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
  vi.resetAllMocks()
  vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('api', 0), component('ui', 1)])
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the components page', () => {
  it('lists the project’s components in order, each with its summary', async () => {
    const page = await render()

    expect(names(page)).toEqual(['api', 'ui'])
    expect(page.get('[data-testid="component-row-api"]').text()).toContain('About api')
  })

  it('groups the components under the folder that holds them', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([
      { ...component('gateway', 0), folder: '/work/vega/gateway' },
      { ...component('ledger', 1), folder: '/work/vega/billing/ledger' },
      { ...component('payouts', 2), folder: '/work/vega/billing/payouts' }
    ])
    const page = await render('/work/vega')

    expect(page.findAll('[data-testid="components-folder"]').map((folder) => folder.text())).toEqual(['billing/2'])
    expect(names(page)).toEqual(['gateway', 'ledger', 'payouts'])
  })

  it('offers a filter only once the list is long, and says when nothing matches', async () => {
    const page = await render()
    expect(page.find('[data-testid="components-filter"]').exists()).toBe(false)

    wrapper?.unmount()
    vi.mocked(api.fetchProjectComponents).mockResolvedValue(Array.from({ length: 8 }, (_, position) => component(`c${position}`, position)))
    const long = await render()
    await long.get('[data-testid="components-filter"]').setValue('c3')
    expect(names(long)).toEqual(['c3'])

    await long.get('[data-testid="components-filter"]').setValue('nothing like it')
    expect(long.get('[data-testid="components-no-match"]').text()).toContain('nothing like it')
  })

  it('invites a scan when the project has no components', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([])
    const page = await render()

    expect(page.get('[data-testid="components-empty"]').text()).toContain('Scan folder')
    expect(page.find('[data-testid="scan-components-empty"]').exists()).toBe(true)
  })

  it('opens a session on the project to scan its folder', async () => {
    const page = await render()

    await page.get('[data-testid="scan-components"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith({ projectId: PROJECT }, expect.objectContaining({ mode: 'SCAN_COMPONENTS' }))
  })

  it('will not scan a project that has no folder', async () => {
    const page = await render(null)

    expect(page.get('[data-testid="scan-components"]').attributes('disabled')).toBeDefined()
  })

  it('reads the open component as a page: its summary, its description drawn and an outline of its headings', async () => {
    const page = await render('/work', 'api')

    expect(page.get('[data-testid="component-summary-read"]').text()).toBe('About api')
    expect(page.get('[data-testid="markdown"]').attributes('data-readonly')).toBeDefined()
    expect(page.get('[data-testid="component-outline"]').text()).toContain('Interfaces')
  })

  it('opens a component with no description for writing, and says so when it is read', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([{ ...component('api', 0), description: '' }])
    const page = await render('/work', 'api')
    expect(page.find('[data-testid="component-summary"]').exists()).toBe(true)

    await page.get('[data-testid="component-mode-read"]').trigger('click')
    await flushPromises()

    expect(page.find('[data-testid="markdown"]').exists()).toBe(false)
    expect(page.get('[data-testid="component-no-description"]').text()).toContain('Scan folder')
  })

  it('adds a component under the folder’s own name when no name is typed', async () => {
    vi.mocked(api.createProjectComponent).mockResolvedValue(component('billing', 2))
    const page = await render()

    await page.get('[data-testid="rail-add-component"]').trigger('click')
    await page.get('[data-testid="new-component-folder"]').setValue('/work/vega/billing/')
    expect((page.get('[data-testid="new-component-name"]').element as HTMLInputElement).placeholder).toBe('billing')
    await page.get('[data-testid="component-composer"]').trigger('submit')
    await flushPromises()

    expect(api.createProjectComponent).toHaveBeenCalledWith(PROJECT, { folder: '/work/vega/billing/', name: 'billing', summary: '' })
    expect(names(page)).toEqual(['api', 'ui', 'billing'])
    expect(page.emitted('open')?.at(-1)).toEqual(['billing'])
  })

  it('will not submit without a folder, and Escape closes the form without a request', async () => {
    const page = await render()
    await page.get('[data-testid="rail-add-component"]').trigger('click')

    expect(page.get('[data-testid="new-component-submit"]').attributes('disabled')).toBeDefined()
    await page.get('[data-testid="component-composer"]').trigger('keydown', { key: 'Escape' })
    await flushPromises()

    expect(page.find('[data-testid="component-composer"]').exists()).toBe(false)
    expect(api.createProjectComponent).not.toHaveBeenCalled()
  })

  it('moves a component and offers no move past either end', async () => {
    vi.mocked(api.reorderProjectComponents).mockResolvedValue([component('ui', 0), component('api', 1)])
    const first = await render('/work', 'api')
    expect(first.get('[data-testid="component-up"]').attributes('disabled')).toBeDefined()
    expect(first.get('[data-testid="component-down"]').attributes('disabled')).toBeUndefined()

    wrapper?.unmount()
    const last = await render('/work', 'ui')
    expect(last.get('[data-testid="component-down"]').attributes('disabled')).toBeDefined()
    await last.get('[data-testid="component-up"]').trigger('click')
    await flushPromises()

    expect(api.reorderProjectComponents).toHaveBeenCalledWith(PROJECT, ['ui', 'api'])
  })

  it('asks before deleting, and names what it removes', async () => {
    vi.mocked(api.deleteProjectComponent).mockResolvedValue()
    const page = await render('/work', 'api')

    await page.get('[data-testid="component-delete"]').trigger('click')
    expect(page.text()).toContain('Delete api?')
    expect(page.text()).toContain('The folder on disk is not touched.')
    expect(api.deleteProjectComponent).not.toHaveBeenCalled()
    await page.findAll('button').find((button) => button.text() === 'Delete component')!.trigger('click')
    await flushPromises()

    expect(api.deleteProjectComponent).toHaveBeenCalledWith('api')
    expect(names(page)).toEqual(['ui'])
  })
})
