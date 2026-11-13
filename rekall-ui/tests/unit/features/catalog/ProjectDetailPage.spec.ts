import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ProjectDetailPage from '@/features/catalog/components/ProjectDetailPage.vue'
import * as sectionsApi from '@/features/project/api/project-sections.api'
import * as componentsApi from '@/features/project/api/project-components.api'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import * as graphsApi from '@/features/graph/api/deployment-graphs.api'
import * as connectionsApi from '@/features/connection/api/connections.api'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'
import { task as workTask } from '../work/work-fixtures'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectConnectionId, ProjectId, ProjectSectionId } from '@/model/branded'

vi.mock('@/features/project/api/project-sections.api')
vi.mock('@/features/project/api/project-components.api')
vi.mock('@/features/connection/api/connections.api')
vi.mock('@/features/graph/api/deployment-graphs.api')
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

const task = workTask('t-1', { projectId: PROJECT, title: 'Build the report' })

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

async function render(tab?: string): Promise<VueWrapper> {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/connections/:id?', name: 'connections', component: { template: '<div />' } }, { path: '/:p(.*)*', component: { template: '<div />' } }] })
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project]
  store.companies = [{ id: project.companyId, name: 'Acme' } as unknown as Company]
  store.tasks = [task]
  wrapper = mountWithHeaderChrome(ProjectDetailPage, {
    props: { id: PROJECT, tab } as never,
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
  vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([])
  vi.mocked(connectionsApi.fetchProjectConnections).mockResolvedValue([])
  vi.mocked(connectionsApi.fetchConnections).mockResolvedValue([])
  vi.mocked(catalogApi.fetchProjectRepository).mockResolvedValue({ isRepository: false } as never)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the project page', () => {
  it('opens on the tasks of the project, as the ledger', async () => {
    const page = await render()

    expect(page.find('[data-testid="project-tasks"]').exists()).toBe(true)
    await page.get('[data-testid="project-tasks"] [data-testid="sub-nav-open"]').trigger('click')
    expect(page.find('[data-testid="work-ledger"]').text()).toContain('Build the report')
    expect(page.get('[data-testid="sub-nav-tasks"]').attributes('aria-selected')).toBe('true')
  })

  it('carries the anchor to copy in its header and no terminal to open', async () => {
    const page = await render()

    expect(page.find('[data-testid="copy-project-anchor"]').exists()).toBe(true)
    expect(page.find('[data-testid="launch-claude-code"]').exists()).toBe(false)
  })

  it('opens the page the route names in place of the tasks', async () => {
    const page = await render('components')
    const workspace = page.get('[data-testid="project-workspace"]')
    expect(workspace.find('[data-testid="project-components"]').exists()).toBe(true)
    expect(workspace.find('[data-testid="project-tasks"]').exists()).toBe(false)

    await page.setProps({ tab: 'connections' })
    await flushPromises()
    expect(workspace.find('[data-testid="project-connections"]').exists()).toBe(true)

    await page.setProps({ tab: 'settings' })
    await flushPromises()
    expect(workspace.find('[data-testid="project-description"]').exists()).toBe(true)
    await page.get('[data-testid="settings-rail-repository"]').trigger('click')
    await flushPromises()
    expect(workspace.find('[data-testid="project-repo-folder"]').exists()).toBe(true)

    await page.setProps({ tab: 'blueprint' })
    await flushPromises()
    expect(workspace.find('[data-testid="blueprint-pane"]').exists()).toBe(true)
  })

  it('loads what the tabs count, and marks connections that lack a credential', async () => {
    const unsigned = {
      id: 'b-1' as ProjectConnectionId,
      connectionId: 'k-1' as ConnectionId,
      projectId: PROJECT,
      kind: 'GITLAB',
      label: 'billing',
      title: 'GitLab',
      baseUrl: 'https://gitlab.example.com',
      kubeContext: null,
      target: 'acme/billing',
      taskManager: false,
      credentialSetAt: null
    } satisfies ProjectConnection
    vi.mocked(connectionsApi.fetchProjectConnections).mockResolvedValue([unsigned])
    const page = await render()

    expect(componentsApi.fetchProjectComponents).toHaveBeenCalledWith(PROJECT)
    const count = (tab: string): string => page.get(`[data-testid="sub-nav-${tab}"] [data-testid="sub-nav-count"]`).text()
    expect(count('pages')).toBe('1')
    expect(count('connections')).toBe('1')
    expect(page.get('[data-testid="sub-nav-connections"] [data-testid="sub-nav-count"]').classes()).toContain('bg-accent')
    expect(page.find('[data-testid="sub-nav-components"] [data-testid="sub-nav-count"]').exists()).toBe(false)
  })

  it('opens the first page from its tab and another from the rail', async () => {
    const glossary = { ...deployment, id: 's-2' as ProjectSectionId, title: 'Glossary', position: 1 }
    vi.mocked(sectionsApi.fetchProjectSections).mockResolvedValue([deployment, glossary])
    const page = await render('pages')
    await flushPromises()

    expect((page.get('[data-testid="section-title"]').element as HTMLInputElement).value).toBe('Deployment')
    expect(page.find('[data-testid="blueprint-pane"]').exists()).toBe(false)

    await page.get('[data-testid="rail-section-s-2"]').trigger('click')
    await flushPromises()
    expect((page.get('[data-testid="section-title"]').element as HTMLInputElement).value).toBe('Glossary')
  })

  it('creates a section from the rail and opens it', async () => {
    const created = { ...deployment, id: 's-2' as ProjectSectionId, title: 'Glossary', bodyMarkdown: '', position: 1 }
    vi.mocked(sectionsApi.createProjectSection).mockResolvedValue(created)
    const page = await render('pages')

    await page.find('[data-testid="rail-new-section"]').trigger('click')
    await page.find('[data-testid="rail-new-title"]').setValue('Glossary')
    await page.find('[data-testid="rail-new-form"]').trigger('submit')
    await flushPromises()

    expect(sectionsApi.createProjectSection).toHaveBeenCalledWith(PROJECT, { title: 'Glossary', bodyMarkdown: '' })
    expect((page.find('[data-testid="section-title"]').element as HTMLInputElement).value).toBe('Glossary')
  })

  it('asks before deleting a section, then stays on the pages with none open', async () => {
    vi.mocked(sectionsApi.deleteProjectSection).mockResolvedValue()
    const page = await render('pages')

    await page.find('[data-testid="section-delete"]').trigger('click')
    expect(page.text()).toContain('Delete Deployment?')
    expect(sectionsApi.deleteProjectSection).not.toHaveBeenCalled()
    const confirm = page.findAll('button').find((button) => button.text() === 'Delete section')!
    await confirm.trigger('click')
    await flushPromises()

    expect(sectionsApi.deleteProjectSection).toHaveBeenCalledWith('s-1')
    expect(page.find('[data-testid="sections-none"]').exists()).toBe(true)
    expect(page.get('[data-testid="sub-nav-pages"]').attributes('aria-selected')).toBe('true')
  })
})
