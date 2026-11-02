import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, ProjectConnection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const COMPANY = 'c-1' as CompanyId

function connection(id: string, over: Partial<Connection> = {}): Connection {
  return {
    id: id as ConnectionId,
    companyIds: [COMPANY],
    rootPath: null,
    kind: 'KUBERNETES',
    label: id,
    title: id,
    baseUrl: null,
    kubeContext: null,
    proxyUrl: null,
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
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod')])
    const store = useConnectionsStore()

    await store.ensureLoaded()
    await store.ensureLoaded()

    expect(store.connectionsOf(COMPANY).map((c) => c.label)).toEqual(['prod'])
    expect(api.fetchConnections).toHaveBeenCalledTimes(1)
  })

  it('creates a connection and then sends its credential, keeping only when it was stored', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('gitlab', { kind: 'GITLAB' }))
    vi.mocked(api.setCredential).mockResolvedValue(
      connection('gitlab', { kind: 'GITLAB', credentialSetAt: '2026-10-21T10:01:00Z' })
    )
    const store = useConnectionsStore()
    await store.load()

    const created = await store.create(
      { kind: 'GITLAB', title: 'GitLab', baseUrl: 'https://gitlab.example.com', companyIds: [COMPANY] },
      { kind: 'GITLAB', token: 'glpat-secret' }
    )

    expect(api.setCredential).toHaveBeenCalledWith('gitlab', { kind: 'GITLAB', token: 'glpat-secret' })
    expect(created.credentialSetAt).toBe('2026-10-21T10:01:00Z')
    expect(JSON.stringify(store.$state)).not.toContain('glpat-secret')
    expect(store.connectionsOf(COMPANY)[0]?.credentialSetAt).toBe('2026-10-21T10:01:00Z')
  })

  it('creates without a credential when none was typed', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([])
    vi.mocked(api.createConnection).mockResolvedValue(connection('prod'))
    const store = useConnectionsStore()

    await store.create({ kind: 'KUBERNETES', title: 'Prod', companyIds: [COMPANY] }, null)

    expect(api.setCredential).not.toHaveBeenCalled()
    expect(store.connectionsOf(COMPANY).map((c) => c.id)).toEqual(['prod'])
  })

  it('forgets a check once the credential is replaced or cleared', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('prod', { credentialSetAt: '2026-10-21T10:00:00Z' })
    ])
    vi.mocked(api.checkConnection).mockResolvedValue({
      ok: false,
      detail: 'Rejected',
      checkedAt: '2026-10-21T10:02:00Z'
    })
    vi.mocked(api.clearCredential).mockResolvedValue(connection('prod'))
    const store = useConnectionsStore()
    await store.load()

    await store.check('prod' as ConnectionId)
    expect(store.checkOf('prod' as ConnectionId)?.detail).toBe('Rejected')
    await store.clearCredential('prod' as ConnectionId)

    expect(store.checkOf('prod' as ConnectionId)).toBeNull()
    expect(store.connectionsOf(COMPANY)[0]?.credentialSetAt).toBeNull()
  })

  it('deletes a connection from its company and its detail', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod'), connection('staging')])
    vi.mocked(api.fetchConnection).mockResolvedValue({ ...connection('prod'), companies: [], projects: [] })
    vi.mocked(api.deleteConnection).mockResolvedValue({ unboundProjects: 2 })
    const store = useConnectionsStore()
    await store.load()
    await store.loadDetail('prod' as ConnectionId)

    const removed = await store.remove(connection('prod'))

    expect(removed.unboundProjects).toBe(2)
    expect(store.connectionsOf(COMPANY).map((c) => c.id)).toEqual(['staging'])
    expect(store.detailOf('prod' as ConnectionId)).toBeNull()
  })

  it('lists the connections shared with a company across the whole workspace', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([
      connection('shared', { companyIds: [COMPANY, 'c-2' as CompanyId] }),
      connection('other', { companyIds: ['c-2' as CompanyId] }),
      connection('nobody', { companyIds: [] })
    ])
    const store = useConnectionsStore()

    await store.load()

    expect(store.connectionsOf(COMPANY).map((c) => c.label)).toEqual(['shared'])
    expect(store.connections).toHaveLength(3)
  })

  it('shares a connection with another set of companies and reads its detail again', async () => {
    const shared = connection('prod', { companyIds: [COMPANY, 'c-2' as CompanyId] })
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod')])
    vi.mocked(api.fetchConnection).mockResolvedValue({ ...shared, companies: [], projects: [] })
    vi.mocked(api.setConnectionCompanies).mockResolvedValue(shared)
    const store = useConnectionsStore()
    await store.load()
    await store.loadDetail('prod' as ConnectionId)

    await store.setCompanies('prod' as ConnectionId, [COMPANY, 'c-2' as CompanyId])

    expect(api.setConnectionCompanies).toHaveBeenCalledWith('prod', [COMPANY, 'c-2'])
    expect(store.connectionById('prod' as ConnectionId)?.companyIds).toEqual([COMPANY, 'c-2'])
    expect(api.fetchConnection).toHaveBeenCalledTimes(2)
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

  function binding(id: string, target: string | null = null, label: string = id): ProjectConnection {
    return {
      id: `b-${label}` as ProjectConnectionId,
      connectionId: id as ConnectionId,
      projectId: PROJECT,
      kind: 'KUBERNETES',
      label,
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

  it('binds a connection twice under two labels and reads the company counts again', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod', { projectCount: 1 })])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    vi.mocked(api.bindConnection)
      .mockResolvedValueOnce(binding('prod', 'apps'))
      .mockResolvedValueOnce(binding('prod', 'billing', 'prod-2'))
    const store = useConnectionsStore()
    await store.load()
    await store.loadProject(PROJECT)

    await store.bind(PROJECT, 'prod' as ConnectionId, { target: 'apps' })
    await store.bind(PROJECT, 'prod' as ConnectionId, { target: 'billing' })

    expect(api.bindConnection).toHaveBeenLastCalledWith(PROJECT, 'prod', { target: 'billing' })
    expect(store.projectConnectionsOf(PROJECT).map((b) => b.label)).toEqual(['prod', 'prod-2'])
    expect(api.fetchConnections).toHaveBeenCalledTimes(3)
  })

  it('saves a binding in place without reading the company again', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod')])
    vi.mocked(api.updateBinding).mockResolvedValue({ ...binding('prod', 'apps'), label: 'apps' })
    const store = useConnectionsStore()
    await store.loadProject(PROJECT)

    await store.updateBinding(PROJECT, 'b-prod' as ProjectConnectionId, { label: 'apps', target: 'apps' })

    expect(store.projectConnectionsOf(PROJECT).map((b) => [b.label, b.target])).toEqual([['apps', 'apps']])
    expect(api.fetchConnections).not.toHaveBeenCalled()
  })

  it('discovers the repositories, then reads the bindings and the company again', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod')])
    vi.mocked(api.fetchProjectConnections).mockResolvedValueOnce([]).mockResolvedValueOnce([binding('prod', 'acme/billing', 'billing')])
    vi.mocked(api.discoverRepositories).mockResolvedValue({ folder: '/work/vega', repositories: [] })
    const store = useConnectionsStore()
    await store.load()
    await store.loadProject(PROJECT)

    const discovery = await store.discover(PROJECT)

    expect(discovery.folder).toBe('/work/vega')
    expect(store.projectConnectionsOf(PROJECT).map((b) => b.label)).toEqual(['billing'])
    expect(api.fetchConnections).toHaveBeenCalledTimes(2)
  })

  it('unbinds a connection from the project only', async () => {
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod'), binding('staging')])
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const store = useConnectionsStore()
    await store.loadProject(PROJECT)

    await store.unbind(PROJECT, 'b-prod' as ProjectConnectionId)

    expect(store.projectConnectionsOf(PROJECT).map((b) => b.label)).toEqual(['staging'])
    expect(api.fetchConnections).not.toHaveBeenCalled()
  })

  it('unbinds a project the connection detail lists by finding its binding first', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod', { projectCount: 1 })])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([binding('prod', 'apps', 'apps')])
    vi.mocked(api.unbindConnection).mockResolvedValue()
    const store = useConnectionsStore()
    await store.load()

    await store.unbindProject('prod' as ConnectionId, {
      projectId: PROJECT,
      companyId: COMPANY,
      label: 'vega',
      title: 'Vega',
      bindingLabel: 'apps',
      target: 'apps'
    })

    expect(api.unbindConnection).toHaveBeenCalledWith(PROJECT, 'b-apps')
    expect(api.fetchConnections).toHaveBeenCalledTimes(2)
  })

  it('has nothing to undo when the binding is already gone', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue([connection('prod')])
    vi.mocked(api.fetchProjectConnections).mockResolvedValue([])
    const store = useConnectionsStore()
    await store.load()

    await store.unbindProject('prod' as ConnectionId, {
      projectId: PROJECT,
      companyId: COMPANY,
      label: 'vega',
      title: 'Vega',
      bindingLabel: 'apps',
      target: null
    })

    expect(api.unbindConnection).not.toHaveBeenCalled()
  })
})
