import { describe, expect, it } from 'vitest'
import { connectionTrunks, trunkMatching, unusedConnections } from '@/features/connection/lib/connection-trunks'
import type { Connection, ProjectConnection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

function connection(id: string, credentialSetAt: string | null = '2026-10-24T10:00:00Z'): Connection {
  return {
    id: id as ConnectionId,
    companyId: 'c-1' as CompanyId,
    kind: 'GITLAB',
    label: id,
    title: `GitLab ${id}`,
    baseUrl: `https://${id}.example.com`,
    kubeContext: null,
    proxyUrl: null,
    credentialSetAt,
    projectCount: 1,
    updatedAt: '2026-10-24T10:00:00Z'
  }
}

function binding(connectionId: string, label: string, target: string | null = null): ProjectConnection {
  return {
    id: `b-${label}` as ProjectConnectionId,
    connectionId: connectionId as ConnectionId,
    projectId: 'p-1' as ProjectId,
    kind: 'GITLAB',
    label,
    title: `GitLab ${connectionId}`,
    baseUrl: `https://${connectionId}.example.com`,
    kubeContext: null,
    target,
    credentialSetAt: null
  }
}

describe('the bindings of a project, by connection', () => {
  it('groups the bindings under their connection, in the order the first of each is listed', () => {
    const trunks = connectionTrunks(
      [binding('acme', 'gateway'), binding('public', 'sdk'), binding('acme', 'orders')],
      [connection('public'), connection('acme')]
    )

    expect(trunks.map((trunk) => [trunk.connectionId, trunk.bindings.map((each) => each.label)])).toEqual([
      ['acme', ['gateway', 'orders']],
      ['public', ['sdk']]
    ])
    expect(trunks[0]).toMatchObject({ title: 'GitLab acme', endpoint: 'acme.example.com', hasCredential: true })
  })

  it('reads the credential from the company connection, and from the binding before the company is loaded', () => {
    expect(connectionTrunks([binding('acme', 'gateway')], [connection('acme', null)])[0]!.hasCredential).toBe(false)
    expect(connectionTrunks([binding('acme', 'gateway')], [])[0]!.hasCredential).toBe(false)
  })

  it('leaves out the connections the project binds, for the rest to be offered', () => {
    const unused = unusedConnections([binding('acme', 'gateway')], [connection('acme'), connection('public')])

    expect(unused.map((each) => each.id)).toEqual(['public'])
  })
})

describe('filtering a connection’s bindings', () => {
  const [trunk] = connectionTrunks(
    [binding('acme', 'gateway', 'shop/gateway'), binding('acme', 'ledger', 'billing/ledger')],
    [connection('acme')]
  )

  it('keeps the bindings whose label or target holds the query, ignoring case', () => {
    expect(trunkMatching(trunk!, 'BILLING')!.bindings.map((each) => each.label)).toEqual(['ledger'])
    expect(trunkMatching(trunk!, 'gate')!.bindings.map((each) => each.label)).toEqual(['gateway'])
  })

  it('keeps the whole connection when its title or host matches, and drops it when nothing does', () => {
    expect(trunkMatching(trunk!, 'acme.example')!.bindings).toHaveLength(2)
    expect(trunkMatching(trunk!, 'jira')).toBeNull()
  })
})
