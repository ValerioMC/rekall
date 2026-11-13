import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ProjectListPage from '@/features/catalog/components/ProjectListPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

const COMPANY = 'c-1' as CompanyId

const project = {
  id: 'p-1' as ProjectId,
  companyId: COMPANY,
  companyName: 'Acme',
  title: 'Vega',
  label: 'vega',
  anchor: 'project:vega',
  status: 'ACTIVE',
  icon: 'folder',
  iconImageVersion: null,
  taskCount: 0,
  updatedAt: '2026-10-05T10:00:00Z'
} as unknown as Project

let pinia: Pinia
let wrapper: VueWrapper | null = null

async function render(logoImageVersion: string | null): Promise<VueWrapper> {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project]
  store.companies = [{ id: COMPANY, name: 'Acme', logoImageVersion } as unknown as Company]
  store.tasks = []
  wrapper = mountWithHeaderChrome(ProjectListPage, { global: { plugins: [pinia, router] } }) as unknown as VueWrapper
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the project list', () => {
  it('shows the logo set on a company beside its name', async () => {
    const page = await render('v1')

    expect(page.find('[data-testid="company-mark-image"] img').attributes('src')).toContain(`/api/companies/${COMPANY}/logo-image`)
  })

  it('shows the company initial when it has no logo', async () => {
    const page = await render(null)

    expect(page.find('[data-testid="company-mark-initial"]').text()).toBe('A')
  })

  it('scrolls on its own when the projects do not fit the screen', async () => {
    const page = await render(null)

    expect(page.find('.bg-canvas.overflow-y-auto').exists()).toBe(true)
  })
})
