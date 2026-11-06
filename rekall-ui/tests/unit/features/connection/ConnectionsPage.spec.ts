import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import * as api from '@/features/connection/api/connections.api'
import ConnectionsPage from '@/features/connection/components/ConnectionsPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'
import { ACME, GLOBEX, aBinding, aDetail, aKindedConnection } from '../../support/connectionFixture'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const ALL = [
  aKindedConnection('jira', 'JIRA', { companyIds: [ACME] }),
  aKindedConnection('platform', 'GITLAB', { rootPath: 'platform', companyIds: [ACME, GLOBEX], projectCount: 1 }),
  aKindedConnection('prod', 'KUBERNETES', { baseUrl: null, kubeContext: 'prod', companyIds: [GLOBEX], credentialSetAt: null })
]

let pinia: Pinia
let router: Router
let wrapper: VueWrapper | null = null

async function render(path = '/connections'): Promise<VueWrapper> {
  router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/connections/:id?', name: 'connections', component: ConnectionsPage }, { path: '/:p(.*)*', component: { template: '<div />' } }]
  })
  await router.push(path)
  const consoleStore = useConsoleStore()
  consoleStore.companies = [
    { id: ACME, name: 'Acme' },
    { id: GLOBEX, name: 'Globex' }
  ] as unknown as Company[]
  consoleStore.projects = [{ id: 'p-billing' as ProjectId, title: 'Billing', companyId: ACME, status: 'ACTIVE' }] as unknown as Project[]
  wrapper = mountWithHeaderChrome(ConnectionsPage, { global: { plugins: [pinia, router] } }) as unknown as VueWrapper
  await flushPromises()
  return wrapper
}

function rowTitles(page: VueWrapper): string[] {
  return page.findAll('[data-testid="connection-row"]').map((row) => row.get('span span').text())
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
  vi.resetAllMocks()
  vi.mocked(api.fetchConnections).mockResolvedValue(ALL)
  vi.mocked(api.fetchConnectionAccess).mockResolvedValue([])
  vi.mocked(api.fetchConnection).mockImplementation(async (id) => {
    const found = ALL.find((connection) => connection.id === id)!
    return aDetail(found, id === 'platform' ? [aBinding('p-billing', ACME, { title: 'Billing', bindingLabel: 'platform', target: 'platform/billing' })] : [])
  })
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the Connections hub', () => {
  it('lists every connection grouped by kind, Kubernetes first, with nothing selected', async () => {
    const page = await render()

    expect(page.findAll('[data-testid="connection-group-name"]').map((heading) => heading.text())).toEqual(['Kubernetes', 'GitLab', 'Jira'])
    expect(rowTitles(page)).toEqual(['The prod', 'The platform', 'The jira'])
    expect(page.find('[data-testid="connection-detail"]').exists()).toBe(false)
  })

  it('narrows the list to the connections shared with the company in the URL', async () => {
    const page = await render(`/connections?company=${GLOBEX}`)

    expect(rowTitles(page)).toEqual(['The prod', 'The platform'])
    expect(page.find('[data-testid="clear-filters"]').exists()).toBe(true)
  })

  it('names the filters when nothing matches and clears them from the empty state', async () => {
    const page = await render(`/connections?company=${GLOBEX}&kind=JIRA`)

    expect(page.get('[data-testid="connections-empty-filtered"]').text()).toContain('company Globex, kind Jira')
    await page.get('[data-testid="empty-clear-filters"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.query).toEqual({})
    expect(rowTitles(page)).toHaveLength(3)
  })

  it('shows the selected connection with who it is shared with, every company, and where it is bound', async () => {
    const page = await render('/connections/platform')

    expect(page.get('[data-testid="detail-title"]').text()).toBe('The platform')
    expect(page.get('[data-testid="connection-root-path-value"]').text()).toBe('platform')
    const companies = page.findAll('[data-testid="sharing-company"]')
    expect(companies.map((company) => company.get('p').text())).toEqual(['Acme', 'Globex'])
    expect(companies[0]?.get('[data-testid="sharing-binding"]').text()).toContain('platform/billing')
    expect(companies[1]?.text()).toContain('This company has no active project to use it.')
    expect(page.get('[data-connection-id="platform"]').attributes('aria-current')).toBe('true')
  })

  it('offers a new connection for the company that has none, instead of a dead end', async () => {
    const page = await render(`/connections?company=${GLOBEX}&kind=JIRA`)

    expect(page.get('[data-testid="connections-empty-filtered"]').text()).toContain('No connections are shared with Globex')
    expect(page.get('[data-testid="empty-new-connection"]').text()).toContain('New connection for Globex')
    expect(page.find('[data-testid="clear-filters"]').exists()).toBe(false)
    expect(page.find('[data-testid="nothing-selected"]').exists()).toBe(false)
  })

  it('says there is nothing to check when no credential is stored, and offers to add one', async () => {
    const page = await render('/connections/prod')

    expect(page.get('[data-testid="credential-none-note"]').text()).toContain('No kubeconfig stored')
    expect(page.find('[data-testid="check-connection"]').exists()).toBe(false)
    expect(page.get('[data-testid="replace-credential"]').text()).toBe('Add kubeconfig')
  })

  it('asks before it unbinds a project and does nothing when asked to cancel', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const page = await render('/connections/platform')

    await page.get('[data-testid="unbind-binding"]').trigger('click')
    expect(page.text()).toContain('Unbind Billing?')
    await page.get('[data-testid="sharing-binding"] button:last-child').trigger('click')

    expect(api.unbindConnection).not.toHaveBeenCalled()
    expect(page.find('[data-testid="unbind-binding"]').exists()).toBe(true)
  })

  it('shows every company in Shared with even when the list is filtered to one', async () => {
    const page = await render(`/connections/platform?company=${ACME}`)

    expect(page.findAll('[data-testid="sharing-company"]')).toHaveLength(2)
  })

  it('says a connection that is gone no longer exists', async () => {
    const page = await render('/connections/vanished')

    expect(page.get('[data-testid="connection-not-found"]').text()).toContain('no longer exists')
  })

  it('starts an empty workspace with the four kinds, and opens the sheet on the one picked', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([])
    const page = await render()

    expect(page.findAll('[data-testid^="start-"]').map((button) => button.attributes('data-testid'))).toEqual([
      'start-KUBERNETES',
      'start-GITLAB',
      'start-GITHUB',
      'start-JIRA'
    ])
    await page.get('[data-testid="start-GITHUB"]').trigger('click')

    expect((page.get('input[data-testid="kind-GITHUB"]').element as HTMLInputElement).checked).toBe(true)
  })

  it('says the list could not be read and tries again', async () => {
    vi.mocked(api.fetchConnections).mockRejectedValueOnce(new Error('offline'))
    const page = await render()

    expect(page.get('[data-testid="connections-error"]').text()).toContain('did not load')
    await page.get('[data-testid="retry-load"]').trigger('click')
    await flushPromises()

    expect(rowTitles(page)).toHaveLength(3)
  })

  it('unbinds a project from the Shared with tree', async () => {
    const binding: ProjectConnection = {
      id: 'b-platform' as ProjectConnectionId,
      connectionId: 'platform' as ConnectionId,
      projectId: 'p-billing' as ProjectId,
      kind: 'GITLAB',
      label: 'platform',
      title: 'The platform',
      baseUrl: null,
      kubeContext: null,
      target: 'platform/billing',
      credentialSetAt: null
    }
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding])
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const page = await render('/connections/platform')

    await page.get('[data-testid="unbind-binding"]').trigger('click')
    await page.get('[data-testid="unbind-binding-confirm"]').trigger('click')
    await flushPromises()

    expect(api.unbindConnection).toHaveBeenCalledWith('p-billing', 'b-platform')
  })

  it('binds a project of the company from the Shared with tree, with the target typed', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.bindConnection).mockResolvedValue({
      id: 'b-new' as ProjectConnectionId,
      connectionId: 'platform' as ConnectionId,
      projectId: 'p-billing' as ProjectId,
      kind: 'GITLAB',
      label: 'platform',
      title: 'The platform',
      baseUrl: null,
      kubeContext: null,
      target: 'platform/billing',
      credentialSetAt: null
    })
    const page = await render('/connections/jira')

    await page.get('[data-testid="bind-project"]').trigger('click')
    expect(page.get('[data-testid="binder-submit"]').attributes('disabled')).toBeDefined()
    await page.get('[data-testid="project-binder"] select').setValue('p-billing')
    await page.get('[data-testid="binder-target"]').setValue(' ENG ')
    await page.get('[data-testid="project-binder"]').trigger('submit')
    await flushPromises()

    expect(api.bindConnection).toHaveBeenCalledWith('p-billing', 'jira', { target: 'ENG' })
    expect(page.find('[data-testid="project-binder"]').exists()).toBe(false)
  })

  it('offers no project to bind where the company has none', async () => {
    const page = await render('/connections/prod')

    expect(page.get('[data-testid="bind-project"]').attributes('disabled')).toBeDefined()
  })

  it('deletes after stating what goes with it, then selects the next row', async () => {
    vi.mocked(api.deleteConnection).mockResolvedValue({ unboundProjects: 1 })
    const page = await render('/connections/platform')

    await page.get('[data-testid="delete-connection"]').trigger('click')
    expect(page.text()).toContain('Unbinds from ')
    const confirm = Array.from(document.body.querySelectorAll('button')).find((button) => button.textContent?.includes('Delete and unbind 1 project'))
    confirm?.click()
    await flushPromises()

    expect(api.deleteConnection).toHaveBeenCalledWith('platform')
    expect(router.currentRoute.value.params.id).toBe('jira')
    expect(rowTitles(page)).toEqual(['The prod', 'The jira'])
  })

  it('keeps the way back to the project that sent the reader here', async () => {
    const page = await render('/connections?company=' + ACME + '&returnTo=/projects/p-billing')

    expect(page.get('[data-testid="back-link"]').text()).toContain('Back to Billing')
    expect(page.get('[data-testid="back-link"]').attributes('href')).toBe('/projects/p-billing')
  })

  it('ignores a way back that leaves the app', async () => {
    const page = await render('/connections?returnTo=https://evil.example.com')

    expect(page.find('[data-testid="back-link"]').exists()).toBe(false)
  })
})
