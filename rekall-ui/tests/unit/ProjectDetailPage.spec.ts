import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ProjectDetailPage from '@/features/catalog/components/ProjectDetailPage.vue'
import * as sectionsApi from '@/features/project/api/project-sections.api'
import * as componentsApi from '@/features/project/api/project-components.api'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from './support/mountWithHeaderChrome'
import type { Company, Project, Task } from '@/features/catalog/model/catalog'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { CompanyId, ProjectId, ProjectSectionId, TaskId } from '@/model/branded'

vi.mock('@/features/project/api/project-sections.api')
vi.mock('@/features/project/api/project-components.api')
vi.mock('@/features/catalog/api/catalog.api', () => ({ fetchProjectRepository: vi.fn().mockResolvedValue({ isRepository: false }) }))

const PROJECT = 'p-1' as ProjectId

const project = {
  id: PROJECT,
  companyId: 'c-1' as CompanyId,
  title: 'Vega',
  label: 'vega',
  anchor: 'project:vega',
  status: 'ACTIVE',
  icon: 'folder',
  iconImageVersion: null,
  description: null,
  blueprintMarkdown: '# Blueprint text',
  repoFolder: null,
  autoCommit: false
} as unknown as Project

const task = {
  id: 't-1' as TaskId,
  projectId: PROJECT,
  label: 'report-builder',
  title: 'Build the report',
  status: 'IN_PROGRESS'
} as unknown as Task

const deployment: ProjectSection = {
  id: 's-1' as ProjectSectionId,
  projectId: PROJECT,
  title: 'Deployment',
  bodyMarkdown: 'Blue-green.',
  position: 0,
  createdAt: '2026-10-05T10:00:00Z',
  updatedAt: '2026-10-05T10:00:00Z'
}

const MarkdownStub = { props: ['modelValue'], template: '<div data-testid="markdown">{{ modelValue }}</div>' }

let pinia: Pinia
let wrapper: VueWrapper | null = null

async function render(): Promise<VueWrapper> {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project]
  store.companies = [{ id: project.companyId, name: 'Acme' } as unknown as Company]
  store.tasks = [task]
  wrapper = mountWithHeaderChrome(ProjectDetailPage, {
    props: { id: PROJECT } as never,
    global: { plugins: [pinia, router], stubs: { AppMarkdownEditor: MarkdownStub } }
  }) as unknown as VueWrapper
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
  vi.resetAllMocks()
  vi.mocked(sectionsApi.fetchProjectSections).mockResolvedValue([deployment])
  vi.mocked(componentsApi.fetchProjectComponents).mockResolvedValue([])
  vi.mocked(catalogApi.fetchProjectRepository).mockResolvedValue({ isRepository: false } as never)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the project page', () => {
  it('opens on the blueprint with the tasks out of sight', async () => {
    const page = await render()

    expect(page.find('[data-testid="blueprint-pane"]').exists()).toBe(true)
    expect(page.find('[data-testid="tasks-drawer"]').exists()).toBe(false)
    expect(page.text()).not.toContain('Build the report')
  })

  it('carries no anchor to copy and no terminal to open in its header', async () => {
    const page = await render()

    expect(page.find('[data-testid="copy-project-anchor"]').exists()).toBe(false)
    expect(page.find('[data-testid="launch-claude-code"]').exists()).toBe(false)
  })

  it('shows the tasks only when asked and closes them again', async () => {
    const page = await render()

    await page.find('[data-testid="show-tasks"]').trigger('click')
    expect(page.find('[data-testid="tasks-drawer"]').text()).toContain('Build the report')

    await page.find('[data-testid="tasks-drawer-close"]').trigger('click')
    await flushPromises()
    expect(page.find('[data-testid="tasks-drawer"]').exists()).toBe(false)
  })

  it('shows the blueprint beside the project’s settings, components among them', async () => {
    const page = await render()
    const inspector = page.get('[data-testid="project-inspector"]')

    expect(componentsApi.fetchProjectComponents).toHaveBeenCalledWith(PROJECT)
    expect(page.get('[data-testid="project-workspace"]').find('[data-testid="blueprint-pane"]').exists()).toBe(true)
    expect(inspector.find('[data-testid="project-components"]').exists()).toBe(true)
    expect(inspector.find('[data-testid="project-description"]').exists()).toBe(true)
    expect(inspector.find('[data-testid="project-repo-folder"]').exists()).toBe(true)
  })

  it('lists the sections in the rail and opens one in place of the blueprint', async () => {
    const page = await render()
    expect(page.find('[data-testid="rail-section-s-1"]').text()).toContain('Deployment')

    await page.find('[data-testid="rail-section-s-1"]').trigger('click')
    await flushPromises()

    expect(page.find('[data-testid="section-pane"]').exists()).toBe(true)
    expect(page.find('[data-testid="blueprint-pane"]').exists()).toBe(false)
    expect((page.find('[data-testid="section-title"]').element as HTMLInputElement).value).toBe('Deployment')
  })

  it('creates a section from the rail and opens it', async () => {
    const created = { ...deployment, id: 's-2' as ProjectSectionId, title: 'Glossary', bodyMarkdown: '', position: 1 }
    vi.mocked(sectionsApi.createProjectSection).mockResolvedValue(created)
    const page = await render()

    await page.find('[data-testid="rail-new-section"]').trigger('click')
    await page.find('[data-testid="rail-new-title"]').setValue('Glossary')
    await page.find('[data-testid="rail-new-form"]').trigger('submit')
    await flushPromises()

    expect(sectionsApi.createProjectSection).toHaveBeenCalledWith(PROJECT, { title: 'Glossary', bodyMarkdown: '' })
    expect((page.find('[data-testid="section-title"]').element as HTMLInputElement).value).toBe('Glossary')
  })

  it('asks before deleting a section, then returns to the blueprint', async () => {
    vi.mocked(sectionsApi.deleteProjectSection).mockResolvedValue()
    const page = await render()
    await page.find('[data-testid="rail-section-s-1"]').trigger('click')
    await flushPromises()

    await page.find('[data-testid="section-delete"]').trigger('click')
    expect(page.text()).toContain('Delete Deployment?')
    expect(sectionsApi.deleteProjectSection).not.toHaveBeenCalled()
    const confirm = page.findAll('button').find((button) => button.text() === 'Delete section')!
    await confirm.trigger('click')
    await flushPromises()

    expect(sectionsApi.deleteProjectSection).toHaveBeenCalledWith('s-1')
    expect(page.find('[data-testid="blueprint-pane"]').exists()).toBe(true)
  })
})
