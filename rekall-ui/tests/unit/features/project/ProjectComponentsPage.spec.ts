import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectComponentsPage from '@/features/project/components/ProjectComponentsPage.vue'
import * as api from '@/features/project/api/project-components.api'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

vi.mock('@/features/project/api/project-components.api')

const PROJECT = 'p-1' as ProjectId

function component(id: string, position: number, name = id): ProjectComponent {
  return {
    id: id as ProjectComponentId,
    projectId: PROJECT,
    name,
    folder: `/work/${id}`,
    description: `About ${id}`,
    position,
    createdAt: '2026-10-08T10:00:00Z',
    updatedAt: '2026-10-08T10:00:00Z'
  }
}

let pinia: Pinia
let wrapper: VueWrapper | null = null

async function render(repoFolder: string | null = null): Promise<VueWrapper> {
  await useProjectComponentsStore().load(PROJECT)
  wrapper = mount(ProjectComponentsPage, {
    props: { projectId: PROJECT, projectTitle: 'Vega', repoFolder },
    global: { plugins: [pinia] }
  })
  await flushPromises()
  return wrapper
}

/** Each component in the tree, top to bottom. */
const names = (page: VueWrapper): string[] =>
  page.findAll('[data-testid="component-row-name"]').map((name) => name.text())

async function openEditor(page: VueWrapper, id: string): Promise<void> {
  await page.get(`[data-testid="component-row-${id}"]`).trigger('click')
}

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
  it('lists the project’s components in order', async () => {
    const page = await render()

    expect(names(page)).toEqual(['api', 'ui'])
  })

  it('draws the components as the tree their folders form under the project folder', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([
      { ...component('gateway', 0), folder: '/work/vega/gateway' },
      { ...component('ledger', 1), folder: '/work/vega/billing/ledger' },
      { ...component('payouts', 2), folder: '/work/vega/billing/payouts' }
    ])
    const page = await render('/work/vega')

    expect(page.get('[data-testid="components-root"]').text()).toBe('vega/')
    expect(page.findAll('[data-testid="components-folder"]').map((folder) => folder.text())).toEqual(['billing/ 2'])
    expect(names(page)).toEqual(['gateway', 'ledger', 'payouts'])
  })

  it('narrows the tree to the components a filter matches, and says when none does', async () => {
    const page = await render()

    await page.get('[data-testid="components-filter"]').setValue('UI')
    expect(names(page)).toEqual(['ui'])

    await page.get('[data-testid="components-filter"]').setValue('nothing like it')
    expect(names(page)).toEqual([])
    expect(page.get('[data-testid="components-no-match"]').text()).toContain('nothing like it')
  })

  it('explains itself and offers the first add when the project has none', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([])
    const page = await render()

    expect(page.find('[data-testid="components-empty"]').exists()).toBe(true)
    expect(page.find('[data-testid="components-filter"]').exists()).toBe(false)
    expect(page.find('[data-testid="add-component"]').exists()).toBe(true)
  })

  it('adds a component under the folder’s own name when no name is typed', async () => {
    vi.mocked(api.createProjectComponent).mockResolvedValue(component('billing', 2))
    const page = await render()

    await page.get('[data-testid="add-component"]').trigger('click')
    await page.get('[data-testid="new-component-folder"]').setValue('/work/vega/billing/')
    expect((page.get('[data-testid="new-component-name"]').element as HTMLInputElement).placeholder).toBe('billing')
    await page.get('[data-testid="component-composer"]').trigger('submit')
    await flushPromises()

    expect(api.createProjectComponent).toHaveBeenCalledWith(PROJECT, {
      folder: '/work/vega/billing/',
      name: 'billing',
      description: ''
    })
    expect(names(page)).toEqual(['api', 'ui', 'billing'])
    expect(page.find('[data-testid="component-composer"]').exists()).toBe(false)
  })

  it('sends a typed name and description as they are, trimmed', async () => {
    vi.mocked(api.createProjectComponent).mockResolvedValue(component('billing', 2))
    const page = await render()

    await page.get('[data-testid="add-component"]').trigger('click')
    await page.get('[data-testid="new-component-folder"]').setValue('/work/billing')
    await page.get('[data-testid="new-component-name"]').setValue('  Billing  ')
    await page.get('[data-testid="new-component-description"]').setValue(' Invoices. ')
    await page.get('[data-testid="component-composer"]').trigger('submit')
    await flushPromises()

    expect(api.createProjectComponent).toHaveBeenCalledWith(PROJECT, {
      folder: '/work/billing',
      name: 'Billing',
      description: 'Invoices.'
    })
  })

  it('will not submit without a folder, and Escape closes the form without a request', async () => {
    const page = await render()
    await page.get('[data-testid="add-component"]').trigger('click')

    expect(page.get('[data-testid="new-component-submit"]').attributes('disabled')).toBeDefined()
    await page.get('[data-testid="component-composer"]').trigger('keydown', { key: 'Escape' })
    await flushPromises()

    expect(page.find('[data-testid="component-composer"]').exists()).toBe(false)
    expect(api.createProjectComponent).not.toHaveBeenCalled()
  })

  it('keeps the form open with what was typed when the server refuses the add', async () => {
    vi.mocked(api.createProjectComponent).mockRejectedValue(new Error('This project already has a component named x'))
    const page = await render()
    await page.get('[data-testid="add-component"]').trigger('click')
    await page.get('[data-testid="new-component-folder"]').setValue('/work/x')

    await page.get('[data-testid="component-composer"]').trigger('submit')
    await flushPromises()

    expect((page.get('[data-testid="new-component-folder"]').element as HTMLInputElement).value).toBe('/work/x')
  })

  it('moves a component and offers no move past either end', async () => {
    vi.mocked(api.reorderProjectComponents).mockResolvedValue([component('ui', 0), component('api', 1)])
    const page = await render()
    await openEditor(page, 'api')
    expect(page.get('[data-testid="component-up"]').attributes('disabled')).toBeDefined()
    expect(page.get('[data-testid="component-down"]').attributes('disabled')).toBeUndefined()
    await page.get('[data-testid="component-close"]').trigger('click')
    await flushPromises()
    await openEditor(page, 'ui')
    expect(page.get('[data-testid="component-down"]').attributes('disabled')).toBeDefined()

    await page.get('[data-testid="component-up"]').trigger('click')
    await flushPromises()

    expect(api.reorderProjectComponents).toHaveBeenCalledWith(PROJECT, ['ui', 'api'])
    expect(names(page)).toEqual(['ui', 'api'])
  })

  it('asks before deleting, and names what it removes', async () => {
    vi.mocked(api.deleteProjectComponent).mockResolvedValue()
    const page = await render()

    await openEditor(page, 'api')
    await page.get('[data-testid="component-delete"]').trigger('click')
    expect(page.text()).toContain('Delete api?')
    expect(page.text()).toContain('The folder on disk is not touched.')
    expect(api.deleteProjectComponent).not.toHaveBeenCalled()
    await page.findAll('button').find((button) => button.text() === 'Delete component')!.trigger('click')
    await flushPromises()

    expect(api.deleteProjectComponent).toHaveBeenCalledWith('api')
    expect(names(page)).toEqual(['ui'])
  })

  it('shows each component as a row and opens one editor at a time', async () => {
    const page = await render()

    expect(page.findAll('[data-testid="component-name"]')).toHaveLength(0)
    await openEditor(page, 'api')
    expect(page.findAll('[data-testid="component-name"]')).toHaveLength(1)
    await openEditor(page, 'ui')
    expect(page.findAll('[data-testid="component-name"]')).toHaveLength(1)
    expect((page.get('[data-testid="component-name"]').element as HTMLInputElement).value).toBe('ui')
  })

  it('lists every component, however many there are', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue(
      Array.from({ length: 8 }, (_, position) => component(`c${position}`, position))
    )
    const page = await render()

    expect(names(page)).toHaveLength(8)
  })
})
