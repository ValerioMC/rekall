import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

export type ConnectionStatus = 'needs-credential' | 'unshared'

export const CONNECTION_STATUSES: readonly ConnectionStatus[] = ['needs-credential', 'unshared']

export const CONNECTION_STATUS_LABEL: Readonly<Record<ConnectionStatus, string>> = {
  'needs-credential': 'Needs credential',
  unshared: 'Not shared with any company'
}

/** What narrows the hub's list; a filter never reorders it. */
export interface ConnectionFilter {
  readonly companyId: CompanyId | null
  readonly kind: ConnectionKind | null
  readonly status: ConnectionStatus | null
}

export const NO_FILTER: ConnectionFilter = { companyId: null, kind: null, status: null }

/** The connections of one kind, as a group of the list. */
export interface KindSection {
  readonly kind: ConnectionKind
  readonly connections: readonly Connection[]
}

function matchesStatus(connection: Connection, status: ConnectionStatus | null): boolean {
  if (status === 'needs-credential') return connection.credentialSetAt === null
  if (status === 'unshared') return connection.companyIds.length === 0
  return true
}

export function filterConnections(connections: readonly Connection[], filter: ConnectionFilter): Connection[] {
  return connections.filter(
    (connection) =>
      (filter.companyId === null || connection.companyIds.includes(filter.companyId)) &&
      (filter.kind === null || connection.kind === filter.kind) &&
      matchesStatus(connection, filter.status)
  )
}

export function activeFilterCount(filter: ConnectionFilter): number {
  return [filter.companyId, filter.kind, filter.status].filter((value) => value !== null).length
}

/** The kinds that have a match, in `KIND_ORDER`, each by label: kinds never interleave. */
export function sectionsByKind(connections: readonly Connection[]): KindSection[] {
  return KIND_ORDER.map((kind) => ({
    kind,
    connections: connections.filter((connection) => connection.kind === kind).sort((a, b) => a.label.localeCompare(b.label))
  })).filter((section) => section.connections.length > 0)
}
