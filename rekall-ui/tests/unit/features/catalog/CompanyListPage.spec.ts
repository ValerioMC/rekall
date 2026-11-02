import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import CompanyListPage from '@/features/catalog/components/CompanyListPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'
import type { Company } from '@/features/catalog/model/catalog'
import type { CompanyId } from '@/model/branded'

const COMPANY = 'c-1' as CompanyId

let pinia: Pinia
let wrapper: VueWrapper | null = null

async function render(logoImageVersion: string | null): Promise<VueWrapper> {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
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
})
