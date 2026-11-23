import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import * as api from '@/features/connection/api/connections.api'
import * as componentsApi from '@/features/project/api/project-components.api'
import ProjectConnectionsPage from '@/features/connection/components/ProjectConnectionsPage.vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Company } from '@/features/catalog/model/catalog'
import type { Project } from '@/features/catalog/model/catalog'
import type { Connection, ConnectionKind, ProjectConnection, RepositoryDiscovery } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')
vi.mock('@/features/catalog/api/catalog.api')
vi.mock('@/features/project/api/project-components.api')

const COMPANY = 'c-1' as CompanyId
const PROJECT = {
  id: 'p-1' as ProjectId,
  label: 'vega',
  title: 'Vega',
  status: 'ACTIVE',
  icon: 'folder',
  iconImageVersion: null,
  description: null,
  blueprintMarkdown: null,
  repoFolder: null,
  autoCommit: false,
  companyId: COMPANY,
  companyName: 'Acme',
  taskCount: 0,
  anchor: 'project:vega',
  updatedAt: '2026-10-21T10:00:00Z'
} as Project

function connection(id: string, over: Partial<Connection> = {}): Connection {
  return {
    id: id as ConnectionId,
    companyIds: [COMPANY],
    rootPath: null,
    kind: 'KUBERNETES',
    label: id,
    title: `The ${id}`,
    baseUrl: null,
    kubeContext: null,
    proxyUrl: null,
    acceptSelfSigned: false,
    credentialSetAt: null,
    projectCount: 0,
    updatedAt: '2026-10-21T10:00:00Z',
    ...over
  }
}

function binding(id: string, credentialSetAt: string | null, kind: ConnectionKind = 'KUBERNETES'): ProjectConnection {
  return {
    id: `b-${id}` as ProjectConnectionId,
    connectionId: id as ConnectionId,
    projectId: PROJECT.id,
    kind,
    label: id,
    title: `The ${id}`,
    baseUrl: null,
    kubeContext: null,
    target: null,
    taskManager: false, credentialSetAt
  }
}

/** The project page loads the connections before it shows this tab, so the test does too. */
async function mountPage(project: Project = PROJECT) {
  const store = useConnectionsStore()
  await Promise.all([store.loadProject(project.id), store.load()])
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/connections/:id?', name: 'connections', component: { template: '<div />' } },
      { path: '/:p(.*)*', component: { template: '<div />' } }
    ]
  })
  useConsoleStore().companies = [{ id: COMPANY, name: 'Acme' }] as unknown as Company[]
  return mount(ProjectConnectionsPage, { props: { project }, global: { plugins: [router] }, attachTo: document.body })
}

const FOUND_BILLING = {
  kind: 'PROJECT',
  path: 'acme/billing',
  name: 'Billing',
  visibility: 'private',
  defaultBranch: 'main',
  lastActivityAt: null,
  subgroupCount: null,
  projectCount: null
} as const

describe('ProjectConnectionsPage', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod'), connection('staging')])
  })

  it('lists the clusters together before the repositories, whatever order the bindings come in', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([
      binding('prod', null),
      binding('gitlab', null, 'GITLAB'),
      binding('staging', null)
    ])
    const wrapper = await mountPage()
    await flushPromises()

    const kinds = wrapper.findAll('[data-testid^="connection-kind-"]')
    expect(kinds.map((kind) => kind.attributes('data-testid'))).toEqual(['connection-kind-KUBERNETES', 'connection-kind-GITLAB'])
    expect(kinds[0]!.findAll('[data-testid^="trunk-"]').map((trunk) => trunk.attributes('data-testid'))).toEqual([
      'trunk-prod',
      'trunk-staging'
    ])
  })

  it('flags a used connection that has no credential and offers it again for another target', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', null)])
    const wrapper = await mountPage()
    await flushPromises()

    const trunk = wrapper.get('[data-testid="trunk-prod"]')
    expect(trunk.find('[data-testid="project-connection-missing"]').exists()).toBe(true)
    expect(trunk.find('[data-testid="add-target"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="use-connection-prod"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="use-connection-staging"]').exists()).toBe(true)
  })

  it('uses a connection of the company the project binds nowhere yet', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.bindConnection).mockResolvedValue(binding('staging', '2026-10-21T10:00:00Z'))
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="use-connection-staging"]').trigger('click')
    await flushPromises()

    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'staging', { target: null })
    expect(wrapper.get('[data-testid="trunk-staging"]').findAll('[data-testid="project-connection"]')).toHaveLength(1)
  })

  it('binds a used connection once more, hanging the new binding under it', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', '2026-10-21T10:00:00Z')])
    vi.mocked(api.bindConnection).mockResolvedValue({
      ...binding('prod', '2026-10-21T10:00:00Z'),
      id: 'b-prod-2' as ProjectConnectionId,
      label: 'prod-2'
    })
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="trunk-prod"] [data-testid="add-target"]').trigger('click')
    await flushPromises()

    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'prod', { target: null })
    expect(wrapper.get('[data-testid="trunk-prod"]').findAll('[data-testid="project-connection"]')).toHaveLength(2)
  })

  it('narrows the bindings to those whose label or target matches the filter', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([
      { ...binding('prod', null), target: 'apps' },
      { ...binding('staging', null), target: 'web' },
      ...['a', 'b', 'c', 'd', 'e', 'f'].map((id) => ({ ...binding(id, null), target: id }))
    ])
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="connections-filter"]').setValue('web')

    expect(wrapper.find('[data-testid="trunk-prod"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="trunk-staging"]').exists()).toBe(true)
  })

  it('saves a target and a label typed in place and unbinds on request', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', '2026-10-21T10:00:00Z')])
    vi.mocked(api.updateBinding)
      .mockResolvedValueOnce({ ...binding('prod', '2026-10-21T10:00:00Z'), target: 'apps' })
      .mockResolvedValueOnce({ ...binding('prod', '2026-10-21T10:00:00Z'), target: 'apps', label: 'prod-apps' })
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const wrapper = await mountPage()
    await flushPromises()

    const target = wrapper.get('[data-testid="connection-target"]')
    await target.setValue(' apps ')
    await target.trigger('blur')
    await flushPromises()
    expect(api.updateBinding).toHaveBeenCalledWith('p-1', 'b-prod', { label: 'prod', target: 'apps' })

    const label = wrapper.get('[data-testid="connection-label"]')
    await label.setValue('prod-apps')
    await label.trigger('blur')
    await flushPromises()
    expect(api.updateBinding).toHaveBeenLastCalledWith('p-1', 'b-prod', { label: 'prod-apps', target: 'apps' })

    await wrapper.get('[data-testid="unbind-connection"]').trigger('click')
    await wrapper.get('[data-testid="unbind-connection-confirm"]').trigger('click')
    await flushPromises()
    expect(api.unbindConnection).toHaveBeenCalledWith('p-1', 'b-prod')
    expect(wrapper.findAll('[data-testid="project-connection"]')).toHaveLength(0)
  })

  it('offers to find repositories only for a project with a folder', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const withoutFolder = await mountPage()
    await flushPromises()

    expect(withoutFolder.find('[data-testid="discover-repositories"]').exists()).toBe(false)
  })

  it('finds the repositories under the folder and shows what each became', async () => {
    const discovery: RepositoryDiscovery = {
      folder: '/work/vega',
      repositories: [
        { folder: '/work/vega/billing', origin: 'gitlab.example.com/acme/billing', outcome: 'BOUND', connectionLabel: 'gitlab', bindingId: 'b-billing' as ProjectConnectionId, label: 'billing', target: 'acme/billing', component: 'billing' },
        { folder: '/work/vega/group/tools', origin: 'github.com/acme/tools', outcome: 'NO_CONNECTION', connectionLabel: null, bindingId: null, label: null, target: null, component: null }
      ]
    }
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.discoverRepositories).mockResolvedValue(discovery)
    vi.mocked(componentsApi.fetchProjectComponents).mockResolvedValue([])
    const wrapper = await mountPage({ ...PROJECT, repoFolder: '/work/vega' })
    await flushPromises()

    await wrapper.get('[data-testid="discover-repositories"]').trigger('click')
    await flushPromises()

    expect(api.discoverRepositories).toHaveBeenCalledWith('p-1')
    expect(componentsApi.fetchProjectComponents).toHaveBeenCalledWith('p-1')
    const rows = wrapper.findAll('[data-testid="discovered-repository"]').map((row) => row.text())
    expect(rows[0]).toContain('billing')
    expect(rows[0]).toContain('Connected')
    expect(rows[1]).toContain('group/tools')
    expect(rows[1]).toContain('No GitLab or GitHub connection for its host')
  })
})

describe('ProjectConnectionsPage as a binding screen', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod'), connection('staging')])
  })

  it('shows the credential state read-only and links each connection to the hub, with the way back', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod', { credentialSetAt: '2026-10-21T10:00:00Z' })])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', '2026-10-21T10:00:00Z')])
    const wrapper = await mountPage()
    await flushPromises()

    const trunk = wrapper.get('[data-testid="trunk-prod"]')
    expect(trunk.find('[data-testid="project-connection-stored"]').text()).toContain('Stored')
    expect(trunk.get('[data-testid="open-in-connections"]').attributes('href')).toBe('/connections/prod?returnTo=/projects/p-1')
    expect(wrapper.find('[data-testid="replace-credential"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="check-connection"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('offers each unused connection with what decides the choice: its scope and its credential', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('platform', { kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', rootPath: 'platform', credentialSetAt: '2026-10-21T10:00:00Z' }),
      connection('clients', { kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', rootPath: 'clients/acme' })
    ])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const wrapper = await mountPage()
    await flushPromises()

    const options = wrapper.findAll('[data-testid="connection-option"]')
    expect(options.map((option) => option.get('[data-testid="connection-option-scope"]').text())).toEqual([
      'gitlab.example.com/clients/acme',
      'gitlab.example.com/platform'
    ])
    expect(options[0]?.text()).toContain('No credential')
    expect(options[1]?.text()).toContain('Stored')
    wrapper.unmount()
  })

  it('opens the path field instead of binding a GitLab connection with no target', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('platform', { kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', rootPath: 'acme', credentialSetAt: '2026-10-21T10:00:00Z' })
    ])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.fetchGitlabChildren).mockResolvedValue([])
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="use-connection-platform"]').trigger('click')
    await flushPromises()

    expect(api.bindConnection).not.toHaveBeenCalled()
    expect(wrapper.find('[data-testid="project-binder"] select').exists()).toBe(false)
    expect(wrapper.find('[data-testid="binder-target"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('binds the GitLab path the instance found, for this project', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('platform', { kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', rootPath: 'acme', credentialSetAt: '2026-10-21T10:00:00Z' })
    ])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.fetchGitlabChildren).mockResolvedValue([])
    vi.mocked(api.fetchTargetCheck).mockResolvedValue({ status: 'FOUND', found: { ...FOUND_BILLING } })
    vi.mocked(api.bindConnection).mockResolvedValue({ ...binding('platform', '2026-10-21T10:00:00Z', 'GITLAB'), target: 'acme/billing' })
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="use-connection-platform"]').trigger('click')
    await wrapper.get('[data-testid="binder-target"]').setValue('acme/billing')
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()
    await wrapper.get('[data-testid="project-binder"]').trigger('submit')
    await flushPromises()

    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'platform', { target: 'acme/billing' })
    expect(wrapper.find('[data-testid="project-binder"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('changes the target of a GitLab binding only to a path the instance found', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('platform', { kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', rootPath: 'acme', credentialSetAt: '2026-10-21T10:00:00Z' })
    ])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([{ ...binding('platform', null, 'GITLAB'), target: 'acme/old' }])
    vi.mocked(api.fetchGitlabChildren).mockResolvedValue([])
    vi.mocked(api.fetchTargetCheck).mockImplementation(async (_id, path) =>
      path === 'acme/billing' ? { status: 'FOUND', found: { ...FOUND_BILLING } } : { status: 'NOT_FOUND', nearest: null }
    )
    vi.mocked(api.updateBinding).mockResolvedValue({ ...binding('platform', null, 'GITLAB'), target: 'acme/billing' })
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="change-gitlab-target"]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-testid="binder-target"]').setValue('acme/bilng')
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()
    expect(wrapper.get('[data-testid="gitlab-target-save"]').attributes('disabled')).toBeDefined()

    await wrapper.get('[data-testid="binder-target"]').setValue('acme/billing')
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()
    await wrapper.get('[data-testid="gitlab-target-editor"]').trigger('submit')
    await flushPromises()

    expect(api.updateBinding).toHaveBeenCalledWith('p-1', 'b-platform', { label: 'platform', target: 'acme/billing' })
    expect(wrapper.find('[data-testid="gitlab-target-editor"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('says no connection is shared with the company, and offers to create one or to share one from the hub', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('elsewhere', { companyIds: ['c-2' as CompanyId] })])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const wrapper = await mountPage()
    await flushPromises()

    const empty = wrapper.get('[data-testid="no-shared-connections"]')
    expect(empty.text()).toContain('No connection is shared with Acme yet')
    expect(empty.get('[data-testid="shared-connections-link"]').attributes('href')).toBe('/connections?company=c-1&returnTo=/projects/p-1')
    await empty.get('[data-testid="empty-new-connection"]').trigger('click')
    expect(document.body.querySelector('[data-testid="connection-sheet"]')).not.toBeNull()
    wrapper.unmount()
  })

  it('creates a connection shared with the project company, then binds it and stays on the tab', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('fresh', { kind: 'KUBERNETES' }))
    vi.mocked(api.bindConnection).mockResolvedValue(binding('fresh', null))
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="new-project-connection"]').trigger('click')
    const sheet = document.body.querySelector('[data-testid="connection-sheet"]')!
    expect(sheet.querySelector('[data-testid="sheet-bind-note"]')?.textContent).toContain('bound to Vega')
    expect(sheet.querySelector('[data-testid="company-picker"]')).toBeNull()
    const title = sheet.querySelector<HTMLInputElement>('[data-testid="connection-title"]')!
    title.value = 'Fresh'
    title.dispatchEvent(new Event('input'))
    sheet.querySelector<HTMLElement>('[data-testid="save-connection"]')!.click()
    await flushPromises()

    expect(api.createConnection).toHaveBeenCalledWith(expect.objectContaining({ companyIds: [COMPANY], title: 'Fresh' }))
    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'fresh', { target: null })
    expect(wrapper.find('[data-testid="trunk-fresh"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('says the connection was created but not bound when the binding fails', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('fresh', { kind: 'KUBERNETES' }))
    vi.mocked(api.bindConnection).mockRejectedValue(new Error('project is archived'))
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="new-project-connection"]').trigger('click')
    const sheet = document.body.querySelector('[data-testid="connection-sheet"]')!
    const title = sheet.querySelector<HTMLInputElement>('[data-testid="connection-title"]')!
    title.value = 'Fresh'
    title.dispatchEvent(new Event('input'))
    sheet.querySelector<HTMLElement>('[data-testid="save-connection"]')!.click()
    await flushPromises()

    const toasts = useToastStore().toasts.map((toast) => toast.message)
    expect(toasts.some((message) => message.includes('The fresh was created but not bound to Vega: project is archived'))).toBe(true)
    expect(useConnectionsStore().connectionById('fresh' as ConnectionId)).not.toBeNull()
    wrapper.unmount()
  })

  it('links to the hub for the rest, carrying the way back', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const wrapper = await mountPage()
    await flushPromises()

    expect(wrapper.get('[data-testid="manage-connections-link"]').attributes('href')).toBe('/connections?company=c-1&returnTo=/projects/p-1')
    wrapper.unmount()
  })
})

describe('ProjectConnectionsPage with a Jira connection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchConnections).mockResolvedValue([])
  })

  function jira(target: string | null, credentialSetAt: string | null): ProjectConnection {
    return { ...binding('jira', credentialSetAt), kind: 'JIRA', baseUrl: 'https://example.atlassian.net', target }
  }

  it('offers the import once the binding has a credential and a project key', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([jira('ENG', '2026-10-21T10:00:00Z')])
    const wrapper = await mountPage()
    await flushPromises()

    expect(wrapper.get('[data-testid="connection-target"]').attributes('aria-label')).toBe('Jira project key for The jira')
    expect(wrapper.find('[data-testid="import-jira-issues"]').exists()).toBe(true)
  })

  it('offers no import without a project key or a credential', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([jira(null, '2026-10-21T10:00:00Z')])
    const withoutKey = await mountPage()
    await flushPromises()
    expect(withoutKey.find('[data-testid="import-jira-issues"]').exists()).toBe(false)

    vi.mocked(api.fetchProjectConnections).mockResolvedValue([jira('ENG', null)])
    const withoutCredential = await mountPage()
    await flushPromises()
    expect(withoutCredential.find('[data-testid="import-jira-issues"]').exists()).toBe(false)
  })
})

describe('ProjectConnectionsPage with a Confluence connection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.useRealTimers()
    vi.mocked(api.fetchConnections).mockResolvedValue([])
    vi.mocked(catalogApi.fetchProjects).mockResolvedValue([])
    vi.mocked(catalogApi.fetchCompanies).mockResolvedValue([])
  })

  function confluence(target: string | null, credentialSetAt: string | null): ProjectConnection {
    return { ...binding('wiki', credentialSetAt), kind: 'CONFLUENCE', baseUrl: 'https://example.atlassian.net/wiki', target }
  }

  it('offers the spaces of the site while the key is typed', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([confluence(null, '2026-10-21T10:00:00Z')])
    vi.mocked(api.fetchTargetSuggestions).mockResolvedValue([{ value: 'DOCS', name: 'Documentation' }])
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="connection-target"]').setValue('do')
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()

    expect(api.fetchTargetSuggestions).toHaveBeenCalledWith('wiki', 'do')
    expect(wrapper.get('[data-testid="connection-target"]').attributes('list')).toBe('target-suggestions-b-wiki')
    expect(wrapper.findAll('datalist option').map((option) => option.attributes('value'))).toEqual(['DOCS'])
    expect(wrapper.find('[data-testid="suggestions-unavailable"]').exists()).toBe(false)
  })

  it('says why the site could not suggest instead of raising a toast on every key', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([confluence(null, '2026-10-21T10:00:00Z')])
    vi.mocked(api.fetchTargetSuggestions).mockRejectedValue(new Error('Confluence rejected the stored credential (HTTP 401).'))
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="connection-target"]').trigger('focus')
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()

    expect(wrapper.get('[data-testid="suggestions-unavailable"]').text()).toContain('HTTP 401')
  })

  it('asks the site for nothing without a credential and offers no filling without a space key', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([confluence('DOCS', null)])
    const withoutCredential = await mountPage()
    await flushPromises()
    await withoutCredential.get('[data-testid="connection-target"]').trigger('focus')
    await new Promise((resolve) => setTimeout(resolve, 300))

    expect(api.fetchTargetSuggestions).not.toHaveBeenCalled()
    expect(withoutCredential.find('[data-testid="fill-from-confluence"]').exists()).toBe(false)

    vi.mocked(api.fetchProjectConnections).mockResolvedValue([confluence(null, '2026-10-21T10:00:00Z')])
    const withoutKey = await mountPage()
    await flushPromises()
    expect(withoutKey.find('[data-testid="fill-from-confluence"]').exists()).toBe(false)
  })

  it('fills the project from the space and says what became of each section', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([confluence('DOCS', '2026-10-21T10:00:00Z')])
    vi.mocked(api.importConfluenceSpace).mockResolvedValue({
      spaceKey: 'DOCS',
      spaceName: 'Documentation',
      pagesRead: 3,
      descriptionWritten: true,
      blueprintWritten: false,
      sections: [
        { title: 'Architecture', pages: 1, outcome: 'created' },
        { title: 'Deployment', pages: 1, outcome: 'kept' }
      ]
    })
    const wrapper = await mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="fill-from-confluence"]').trigger('click')
    await wrapper.get('[data-testid="confluence-replace"] input').setValue(true)
    await wrapper.get('[data-testid="fill-project"]').trigger('click')
    await flushPromises()

    expect(api.importConfluenceSpace).toHaveBeenCalledWith('p-1', 'b-wiki', true)
    const result = wrapper.get('[data-testid="confluence-import-result"]').text()
    expect(result).toContain('Read 3 pages of Documentation.')
    expect(result).toContain('Description: written')
    expect(result).toContain('Blueprint: kept')
    expect(wrapper.findAll('[data-testid="confluence-section"]').map((row) => row.text())).toEqual([
      expect.stringContaining('Architecture'),
      expect.stringContaining('Deployment')
    ])
  })
})
