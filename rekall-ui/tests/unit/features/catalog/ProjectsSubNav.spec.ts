import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ProjectsSubNav from '@/features/catalog/components/ProjectsSubNav.vue'
import * as connectionsApi from '@/features/connection/api/connections.api'
import { aConnection } from '../../support/connectionFixture'
import { useConsoleStore } from '@/stores/console.store'
import type { Company } from '@/features/catalog/model/catalog'
import { project } from '../work/work-fixtures'

vi.mock('@/features/connection/api/connections.api')

describe('the Projects sub-navigation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(connectionsApi.fetchConnections).mockResolvedValue([aConnection('hub')])
  })

  it('links the three screens and counts what each holds, archived projects left out', async () => {
    const store = useConsoleStore()
    store.projects = [project('a', 'A'), project('b', 'B', { status: 'ARCHIVED' })]
    store.companies = [{ id: 'c1', name: 'Acme' } as unknown as Company]
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
    const wrapper = mount(ProjectsSubNav, { props: { active: 'companies' }, global: { plugins: [router] } })
    await flushPromises()

    const tabs = wrapper.findAll('[role="tab"]')
    expect(tabs.map((tab) => tab.text())).toEqual(['Projects 1', 'Companies 1', 'Connections 1'])
    expect(tabs.map((tab) => tab.attributes('href'))).toEqual(['/projects', '/companies', '/connections'])
    expect(tabs[1]!.attributes('aria-selected')).toBe('true')
  })
})
