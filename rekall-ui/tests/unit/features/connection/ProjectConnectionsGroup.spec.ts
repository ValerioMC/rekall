import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ProjectConnectionsGroup from '@/features/connection/components/ProjectConnectionsGroup.vue'
import type { Project } from '@/features/catalog/model/catalog'
import type { Connection, ProjectConnection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

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

function connection(id: string): Connection {
  return {
    id: id as ConnectionId,
    companyId: COMPANY,
    kind: 'KUBERNETES',
    label: id,
    title: `The ${id}`,
    baseUrl: null,
    kubeContext: null,
    credentialSetAt: null,
    projectCount: 0,
    updatedAt: '2026-10-21T10:00:00Z'
  }
}

function binding(id: string, credentialSetAt: string | null): ProjectConnection {
  return {
    connectionId: id as ConnectionId,
    projectId: PROJECT.id,
    kind: 'KUBERNETES',
    label: id,
    title: `The ${id}`,
    baseUrl: null,
    kubeContext: null,
    target: null,
    credentialSetAt
  }
}

function mountGroup() {
  return mount(ProjectConnectionsGroup, {
    props: { project: PROJECT },
    global: { stubs: { RouterLink: { template: '<a><slot /></a>' } } }
  })
}

describe('ProjectConnectionsGroup', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([connection('prod'), connection('staging')])
  })

  it('flags a used connection that has no credential and offers only the unused ones', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', null)])
    const wrapper = mountGroup()
    await flushPromises()

    expect(wrapper.find('[data-testid="project-connection-missing"]').exists()).toBe(true)
    const options = wrapper.findAll('[data-testid="connection-picker"] option').map((option) => option.text())
    expect(options.some((text) => text.includes('staging'))).toBe(true)
    expect(options.some((text) => text.includes('The prod'))).toBe(false)
  })

  it('binds the connection chosen in the picker', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.bindConnection).mockResolvedValue(binding('staging', '2026-10-21T10:00:00Z'))
    const wrapper = mountGroup()
    await flushPromises()

    await wrapper.get('[data-testid="connection-picker"] select').setValue('staging')
    await wrapper.get('[data-testid="bind-connection"]').trigger('click')
    await flushPromises()

    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'staging', null)
    expect(wrapper.findAll('[data-testid="project-connection"]')).toHaveLength(1)
  })

  it('saves a target typed in place and unbinds on request', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', '2026-10-21T10:00:00Z')])
    vi.mocked(api.bindConnection).mockResolvedValue({
      ...binding('prod', '2026-10-21T10:00:00Z'),
      target: 'apps'
    })
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const wrapper = mountGroup()
    await flushPromises()

    const target = wrapper.get('[data-testid="connection-target"]')
    await target.setValue(' apps ')
    await target.trigger('blur')
    await flushPromises()
    expect(api.bindConnection).toHaveBeenCalledWith('p-1', 'prod', 'apps')

    await wrapper.get('[data-testid="unbind-connection"]').trigger('click')
    await flushPromises()
    expect(api.unbindConnection).toHaveBeenCalledWith('p-1', 'prod')
    expect(wrapper.findAll('[data-testid="project-connection"]')).toHaveLength(0)
  })
})
