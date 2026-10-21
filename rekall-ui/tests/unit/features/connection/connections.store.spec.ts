import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, ProjectConnection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const COMPANY = 'c-1' as CompanyId

function connection(id: string, over: Partial<Connection> = {}): Connection {
  return {
    id: id as ConnectionId,
    companyId: COMPANY,
    kind: 'KUBERNETES',
    label: id,
    title: id,
    baseUrl: null,
    kubeContext: null,
    credentialSetAt: null,
    projectCount: 0,
    updatedAt: '2026-10-21T10:00:00Z',
    ...over
  }
}

describe('the connections store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads a company once opened', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([connection('prod')])
    const store = useConnectionsStore()

    await store.ensureLoaded(COMPANY)
    await store.ensureLoaded(COMPANY)

    expect(store.connectionsOf(COMPANY).map((c) => c.label)).toEqual(['prod'])
    expect(api.fetchCompanyConnections).toHaveBeenCalledTimes(1)
  })

  it('creates a connection and then sends its credential, keeping only when it was stored', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('gitlab', { kind: 'GITLAB' }))
    vi.mocked(api.setCredential).mockResolvedValue(
      connection('gitlab', { kind: 'GITLAB', credentialSetAt: '2026-10-21T10:01:00Z' })
    )
    const store = useConnectionsStore()
    await store.load(COMPANY)

    const created = await store.create(
      COMPANY,
      { kind: 'GITLAB', title: 'GitLab', baseUrl: 'https://gitlab.example.com' },
      { kind: 'GITLAB', token: 'glpat-secret' }
    )

    expect(api.setCredential).toHaveBeenCalledWith('gitlab', { kind: 'GITLAB', token: 'glpat-secret' })
    expect(created.credentialSetAt).toBe('2026-10-21T10:01:00Z')
    expect(JSON.stringify(store.$state)).not.toContain('glpat-secret')
    expect(store.connectionsOf(COMPANY)[0]?.credentialSetAt).toBe('2026-10-21T10:01:00Z')
  })

  it('creates without a credential when none was typed', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('prod'))
    const store = useConnectionsStore()

    await store.create(COMPANY, { kind: 'KUBERNETES', title: 'Prod' }, null)

    expect(api.setCredential).not.toHaveBeenCalled()
    expect(store.connectionsOf(COMPANY).map((c) => c.id)).toEqual(['prod'])
  })

  it('forgets a check once the credential is replaced or cleared', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([
      connection('prod', { credentialSetAt: '2026-10-21T10:00:00Z' })
    ])
    vi.mocked(api.checkConnection).mockResolvedValue({
      ok: false,
      detail: 'Rejected',
      checkedAt: '2026-10-21T10:02:00Z'
    })
    vi.mocked(api.clearCredential).mockResolvedValue(connection('prod'))
    const store = useConnectionsStore()
    await store.load(COMPANY)

    await store.check('prod' as ConnectionId)
    expect(store.checkOf('prod' as ConnectionId)?.detail).toBe('Rejected')
    await store.clearCredential('prod' as ConnectionId)

    expect(store.checkOf('prod' as ConnectionId)).toBeNull()
    expect(store.connectionsOf(COMPANY)[0]?.credentialSetAt).toBeNull()
  })

  it('deletes a connection from its company and its detail', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([connection('prod'), connection('staging')])
    vi.mocked(api.fetchConnection).mockResolvedValue({ ...connection('prod'), projects: [] })
    vi.mocked(api.deleteConnection).mockResolvedValue({ unboundProjects: 2 })
    const store = useConnectionsStore()
    await store.load(COMPANY)
    await store.loadDetail('prod' as ConnectionId)

    const removed = await store.remove(connection('prod'))

    expect(removed.unboundProjects).toBe(2)
    expect(store.connectionsOf(COMPANY).map((c) => c.id)).toEqual(['staging'])
    expect(store.detailOf('prod' as ConnectionId)).toBeNull()
  })

  it('pages the access log and knows when the oldest request was read', async () => {
    const first = Array.from({ length: 50 }, (_, index) => ({
      id: `a-${index}`,
      accessedAt: '2026-10-21T10:00:00Z',
      granted: true,
      reason: null,
      terminalId: 't-1',
      taskId: null,
      taskLabel: null,
      taskTitle: null
    }))
    vi.mocked(api.fetchConnectionAccess)
      .mockResolvedValueOnce(first)
      .mockResolvedValueOnce([{ ...first[0]!, id: 'a-50' }])
    const store = useConnectionsStore()

    await store.loadAccess('prod' as ConnectionId)
    expect(store.hasMoreAccess('prod' as ConnectionId)).toBe(true)
    await store.loadMoreAccess('prod' as ConnectionId)

    expect(store.accessOf('prod' as ConnectionId)).toHaveLength(51)
    expect(api.fetchConnectionAccess).toHaveBeenLastCalledWith('prod', 50, 50)
    expect(store.hasMoreAccess('prod' as ConnectionId)).toBe(false)
  })
})

describe('a project in the connections store', () => {
  const PROJECT = 'p-1' as ProjectId

  function binding(id: string, target: string | null = null): ProjectConnection {
    return {
      connectionId: id as ConnectionId,
      projectId: PROJECT,
      kind: 'KUBERNETES',
      label: id,
      title: id,
      baseUrl: null,
      kubeContext: null,
      target,
      credentialSetAt: null
    }
  }

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('binds a connection, sets its target in place and reads the company counts again', async () => {
    vi.mocked(api.fetchCompanyConnections).mockResolvedValue([connection('prod', { projectCount: 1 })])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.bindConnection)
      .mockResolvedValueOnce(binding('prod'))
      .mockResolvedValueOnce(binding('prod', 'apps'))
    const store = useConnectionsStore()
    await store.load(COMPANY)
    await store.loadProject(PROJECT)

    await store.bind(PROJECT, COMPANY, 'prod' as ConnectionId, null)
    await store.bind(PROJECT, COMPANY, 'prod' as ConnectionId, 'apps')

    expect(store.projectConnectionsOf(PROJECT)).toEqual([binding('prod', 'apps')])
    expect(api.fetchCompanyConnections).toHaveBeenCalledTimes(3)
  })

  it('unbinds a connection from the project only', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod'), binding('staging')])
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const store = useConnectionsStore()
    await store.loadProject(PROJECT)

    await store.unbind(PROJECT, COMPANY, 'prod' as ConnectionId)

    expect(store.projectConnectionsOf(PROJECT).map((b) => b.label)).toEqual(['staging'])
    expect(api.fetchCompanyConnections).not.toHaveBeenCalled()
  })
})
