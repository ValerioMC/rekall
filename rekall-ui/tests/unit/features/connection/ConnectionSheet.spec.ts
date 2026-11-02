import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ConnectionSheet from '@/features/connection/components/ConnectionSheet.vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'
import { ACME, GLOBEX, aConnection } from '../../support/connectionFixture'

vi.mock('@/features/connection/api/connections.api')

const COMPANIES = [
  { id: ACME, name: 'Acme' },
  { id: GLOBEX, name: 'Globex' }
]

function mountSheet(connection: Connection | null, initialKind: ConnectionKind | null = null, initialCompanyIds: CompanyId[] = []) {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  return mount(ConnectionSheet, {
    props: { connection, initialKind, initialCompanyIds, companies: COMPANIES },
    global: { plugins: [router] },
    attachTo: document.body
  })
}

describe('ConnectionSheet', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('asks for a root path on GitLab and GitHub only, with a worked example', async () => {
    const wrapper = mountSheet(null, 'GITLAB')

    expect(wrapper.find('[data-testid="connection-root-path"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('Leave empty to cover the whole instance')
    await wrapper.get('[data-testid="kind-GITHUB"]').setValue(true)
    expect(wrapper.text()).toContain('Account or organisation')
    await wrapper.get('[data-testid="kind-KUBERNETES"]').setValue(true)
    expect(wrapper.find('[data-testid="connection-root-path"]').exists()).toBe(false)
    await wrapper.get('[data-testid="kind-JIRA"]').setValue(true)
    expect(wrapper.find('[data-testid="connection-root-path"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('lists the kinds in the order every list uses', () => {
    const wrapper = mountSheet(null)

    expect(wrapper.findAll('input[name="connection-kind"]').map((input) => input.attributes('data-testid'))).toEqual([
      'kind-KUBERNETES',
      'kind-GITLAB',
      'kind-GITHUB',
      'kind-JIRA'
    ])
    wrapper.unmount()
  })

  it('creates a GitLab connection with its root path and the companies picked, starting from the company in view', async () => {
    vi.mocked(api.createConnection).mockResolvedValue(aConnection('platform', { rootPath: 'platform/backend' }))
    const wrapper = mountSheet(null, 'GITLAB', [ACME])
    await wrapper.get('[data-testid="connection-title"]').setValue('Platform')
    await wrapper.get('[data-testid="connection-url"]').setValue('https://gitlab.example.com')
    await wrapper.get('[data-testid="connection-root-path"]').setValue(' platform/backend ')
    const boxes = wrapper.findAll('[data-testid="company-picker"] input[type="checkbox"]')
    expect((boxes[0]?.element as HTMLInputElement).checked).toBe(true)
    await boxes[1]?.setValue(true)
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()

    expect(api.createConnection).toHaveBeenCalledWith({
      kind: 'GITLAB',
      title: 'Platform',
      label: 'platform',
      baseUrl: 'https://gitlab.example.com',
      rootPath: 'platform/backend',
      companyIds: [ACME, GLOBEX]
    })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    wrapper.unmount()
  })

  it('edits without the company checklist, pointing to the Shared with panel instead', async () => {
    const wrapper = mountSheet(aConnection('platform', { rootPath: 'platform' }))

    expect(wrapper.find('[data-testid="company-picker"]').exists()).toBe(false)
    expect(wrapper.find('input[name="connection-kind"]').exists()).toBe(false)
    await wrapper.get('[data-testid="sheet-edit-companies"]').trigger('click')
    expect(wrapper.emitted('editCompanies')).toHaveLength(1)
    wrapper.unmount()
  })

  it('clears the root path of an edited connection when it is emptied', async () => {
    vi.mocked(api.updateConnection).mockResolvedValue(aConnection('platform'))
    const wrapper = mountSheet(aConnection('platform', { rootPath: 'platform' }))

    await wrapper.get('[data-testid="connection-root-path"]').setValue('')
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()

    expect(api.updateConnection).toHaveBeenCalledWith('platform', expect.objectContaining({ rootPath: '' }))
    wrapper.unmount()
  })

  it('refuses a missing title in a summary that takes focus, and a label another connection holds', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([aConnection('taken', { title: 'Taken one' })])
    const wrapper = mountSheet(null, 'GITLAB')
    const { useConnectionsStore } = await import('@/features/connection/store/connections.store')
    await useConnectionsStore().load()

    await wrapper.get('[data-testid="connection-url"]').setValue('https://gitlab.example.com')
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()
    expect(wrapper.get('[data-testid="error-summary"]').text()).toContain('A title is required.')
    expect(document.activeElement).toBe(wrapper.get('[data-testid="error-summary"]').element)

    await wrapper.get('[data-testid="connection-title"]').setValue('Taken')
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()
    expect(wrapper.get('[data-testid="error-summary"]').text()).toContain('The label taken is already used by Taken one.')
    expect(api.createConnection).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
