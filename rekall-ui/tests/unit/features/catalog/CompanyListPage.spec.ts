import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import CompanyListPage from '@/features/catalog/components/CompanyListPage.vue'
import * as connectionsApi from '@/features/connection/api/connections.api'
import { aConnection } from '../../support/connectionFixture'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'
import { project, task } from '../project/project-task-fixtures'
import type { Company } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const COMPANY = 'c-1' as CompanyId

let pinia: Pinia
let wrapper: VueWrapper | null = null
let router: ReturnType<typeof createRouter>

async function render(logoImageVersion: string | null): Promise<VueWrapper> {
  router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/connections/:id?', name: 'connections', component: { template: '<div />' } }, { path: '/:p(.*)*', component: { template: '<div />' } }] })
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = []
  store.tasks = []
  store.companies = [
    { id: COMPANY, name: 'Acme', description: '', projectCount: 0, taskCount: 0, updatedAt: '2026-10-05T10:00:00Z', logoImageVersion } as unknown as Company
  ]
  wrapper = mountWithHeaderChrome(CompanyListPage, { global: { plugins: [pinia, router] } }) as unknown as VueWrapper
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
  vi.resetAllMocks()
  vi.mocked(connectionsApi.fetchConnections).mockResolvedValue([])
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the company list', () => {
  it('shows the logo set on a company beside its name', async () => {
    const page = await render('v1')

    expect(page.find('[data-testid="company-mark-image"] img').attributes('src')).toContain(`/api/companies/${COMPANY}/logo-image`)
  })

  it('shows the company initial when it has no logo', async () => {
    const page = await render(null)

    expect(page.find('[data-testid="company-mark-initial"]').text()).toBe('A')
  })

  it('counts the connections shared with the company in a visible link to the hub filtered to it', async () => {
    vi.mocked(connectionsApi.fetchConnections).mockResolvedValue([
      aConnection('platform', { companyIds: [COMPANY] }),
      aConnection('hub', { companyIds: [COMPANY, 'c-2' as CompanyId] }),
      aConnection('elsewhere', { companyIds: ['c-2' as CompanyId] })
    ])
    const page = await render(null)
    const link = page.get('[data-testid="company-connections"]')

    expect(link.text()).toBe('2 connections')
    await link.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe(`/connections?company=${COMPANY}`)
  })

  it('says there are no connections when none is shared with the company', async () => {
    const page = await render(null)

    expect(page.get('[data-testid="company-connections"]').text()).toBe('No connections')
  })

  it('lists the selected company\'s projects as ledger rows that open the project and cannot be deleted from here', async () => {
    const page = await render(null)
    useConsoleStore().projects = [project('vega', 'Vega', { companyId: COMPANY })]
    await flushPromises()

    expect(page.get('[data-testid="company-detail"]').text()).toContain('Vega')
    expect(page.find('[data-testid="delete-project"]').exists()).toBe(false)
    await page.get('[data-testid="project-card"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/projects/vega')
  })

  it('says how many tasks of the company wait for review', async () => {
    const page = await render(null)
    const store = useConsoleStore()
    store.projects = [project('vega', 'Vega', { companyId: COMPANY, taskCount: 1 })]
    store.tasks = [task('t-1', { projectId: 'vega' as ProjectId, reviewActive: true, reviewState: 'CLAIMED' })]
    await flushPromises()

    expect(page.get('[data-testid="company-needs"]').text()).toBe('1 to review')
  })
})
