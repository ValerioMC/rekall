import { describe, expect, it } from 'vitest'
import {
  NO_FILTER,
  activeFilterCount,
  filterConnections,
  sectionsByKind
} from '@/features/connection/lib/connection-filter'
import { filterFromQuery, internalPath, queryOf, returnLabelOf } from '@/features/connection/lib/connection-route'
import { changeAnnouncement } from '@/features/connection/lib/connection-changes'
import { ACME, GLOBEX, aConnection, aKindedConnection } from '../../support/connectionFixture'

const ALL = [
  aKindedConnection('jira', 'JIRA'),
  aKindedConnection('hub', 'GITHUB', { companyIds: [ACME, GLOBEX] }),
  aKindedConnection('zeta', 'GITLAB', { credentialSetAt: null }),
  aKindedConnection('alpha', 'GITLAB', { companyIds: [] }),
  aKindedConnection('prod', 'KUBERNETES', { companyIds: [GLOBEX] })
]

describe('grouping the connections', () => {
  it('orders the kinds Kubernetes, GitLab, GitHub, Jira and never interleaves them', () => {
    const sections = sectionsByKind(ALL)

    expect(sections.map((section) => section.kind)).toEqual(['KUBERNETES', 'GITLAB', 'GITHUB', 'JIRA'])
    expect(sections[1]?.connections.map((connection) => connection.label)).toEqual(['alpha', 'zeta'])
  })

  it('leaves out the kinds that have no match', () => {
    expect(sectionsByKind(ALL.filter((connection) => connection.kind === 'JIRA')).map((s) => s.kind)).toEqual(['JIRA'])
  })
})

describe('filtering the connections', () => {
  it('keeps the connections shared with the company', () => {
    const labels = filterConnections(ALL, { ...NO_FILTER, companyId: GLOBEX }).map((c) => c.label)

    expect(labels).toEqual(['hub', 'prod'])
  })

  it('keeps the ones that need a credential, or that no company can use', () => {
    expect(filterConnections(ALL, { ...NO_FILTER, status: 'needs-credential' }).map((c) => c.label)).toEqual(['zeta'])
    expect(filterConnections(ALL, { ...NO_FILTER, status: 'unshared' }).map((c) => c.label)).toEqual(['alpha'])
  })

  it('combines the filters and counts the ones set', () => {
    const filter = { companyId: ACME, kind: 'GITLAB' as const, status: null }

    expect(filterConnections(ALL, filter).map((c) => c.label)).toEqual(['zeta'])
    expect(activeFilterCount(filter)).toBe(2)
    expect(activeFilterCount(NO_FILTER)).toBe(0)
  })
})

describe('the hub in the URL', () => {
  it('reads the filters and drops values it does not know', () => {
    expect(filterFromQuery({ company: ACME, kind: 'GITLAB', status: 'unshared' })).toEqual({
      companyId: ACME,
      kind: 'GITLAB',
      status: 'unshared'
    })
    expect(filterFromQuery({ kind: 'SVN', status: 'broken' })).toEqual(NO_FILTER)
  })

  it('writes only what is set, keeping the way back', () => {
    expect(queryOf({ ...NO_FILTER, companyId: ACME }, '/projects/p-1')).toEqual({ company: ACME, returnTo: '/projects/p-1' })
    expect(queryOf(NO_FILTER, null)).toEqual({})
  })

  it('follows a way back only inside the app', () => {
    expect(internalPath('/projects/p-1')).toBe('/projects/p-1')
    expect(internalPath('https://evil.example.com')).toBeNull()
    expect(internalPath('//evil.example.com')).toBeNull()
    expect(internalPath(null)).toBeNull()
  })

  it('names the way back by the project it came from', () => {
    const titles = new Map([['p-1', 'Billing']])

    expect(returnLabelOf('/projects/p-1', titles)).toBe('Billing')
    expect(returnLabelOf('/companies', titles)).toBe('Companies')
  })
})

describe('announcing a change', () => {
  it('names the connection that changed', () => {
    const before = [aConnection('a'), aConnection('b')]
    const after = [aConnection('a'), aConnection('b', { updatedAt: '2026-10-22T10:00:00Z' })]

    expect(changeAnnouncement(before, after)).toBe('Connection The b updated')
  })

  it('says a connection went when one is missing', () => {
    expect(changeAnnouncement([aConnection('a'), aConnection('b')], [aConnection('a')])).toBe('A connection was removed')
  })
})
