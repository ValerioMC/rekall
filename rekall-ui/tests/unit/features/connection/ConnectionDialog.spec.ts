import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ConnectionDialog from '@/features/connection/components/ConnectionDialog.vue'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const COMPANY = 'c-1' as CompanyId

const JIRA: Connection = {
  id: 'jira' as ConnectionId,
  companyId: COMPANY,
  kind: 'JIRA',
  label: 'jira',
  title: 'Acme Jira',
  baseUrl: 'https://example.atlassian.net',
  kubeContext: null,
  proxyUrl: 'http://proxy.example.com:8080',
  credentialSetAt: null,
  projectCount: 0,
  updatedAt: '2026-10-22T10:00:00Z'
}

function mountDialog(connection: Connection | null, initialKind: ConnectionKind | null = null) {
  return mount(ConnectionDialog, { props: { companyId: COMPANY, connection, initialKind }, attachTo: document.body })
}

describe('ConnectionDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('asks for a proxy only on a Jira connection', () => {
    const gitlab = mountDialog(null, 'GITLAB')
    const jira = mountDialog(null, 'JIRA')

    expect(gitlab.find('[data-testid="connection-proxy"]').exists()).toBe(false)
    expect(jira.find('[data-testid="connection-proxy"]').exists()).toBe(true)
    gitlab.unmount()
    jira.unmount()
  })

  it('creates a Jira connection with the proxy typed', async () => {
    vi.mocked(api.createConnection).mockResolvedValue(JIRA)
    const wrapper = mountDialog(null, 'JIRA')

    await wrapper.get('[data-testid="connection-title"]').setValue('Acme Jira')
    await wrapper.get('[data-testid="connection-url"]').setValue('https://example.atlassian.net')
    await wrapper.get('[data-testid="connection-proxy"]').setValue(' http://proxy.example.com:8080 ')
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()

    expect(api.createConnection).toHaveBeenCalledWith(COMPANY, {
      kind: 'JIRA',
      title: 'Acme Jira',
      label: 'acme-jira',
      baseUrl: 'https://example.atlassian.net',
      proxyUrl: 'http://proxy.example.com:8080'
    })
    wrapper.unmount()
  })

  it('clears the proxy of an edited connection when it is emptied', async () => {
    vi.mocked(api.updateConnection).mockResolvedValue({ ...JIRA, proxyUrl: null })
    const wrapper = mountDialog(JIRA)

    expect((wrapper.get('[data-testid="connection-proxy"]').element as HTMLInputElement).value).toBe(
      'http://proxy.example.com:8080'
    )
    await wrapper.get('[data-testid="connection-proxy"]').setValue('')
    await wrapper.get('[data-testid="save-connection"]').trigger('click')
    await flushPromises()

    expect(api.updateConnection).toHaveBeenCalledWith('jira', expect.objectContaining({ proxyUrl: '' }))
    wrapper.unmount()
  })
})
