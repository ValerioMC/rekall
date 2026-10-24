import { endpointOf } from '@/features/connection/lib/connection-endpoint'
import type { Connection, ConnectionKind, ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/**
 * One company connection a project uses and every binding it hangs off it. The credential is the
 * connection's, so all the bindings of a trunk have it or lack it together.
 */
export interface ConnectionTrunk {
  readonly connectionId: ConnectionId
  readonly kind: ConnectionKind
  readonly title: string
  readonly endpoint: string
  readonly hasCredential: boolean
  readonly bindings: readonly ProjectConnection[]
}

/** The trunks of one kind of system, as the project's Connections page sets them apart. */
export interface ConnectionKindGroup {
  readonly kind: ConnectionKind
  readonly trunks: readonly ConnectionTrunk[]
}

const KIND_ORDER: readonly ConnectionKind[] = ['KUBERNETES', 'GITLAB', 'JIRA']

function trunkOf(bindings: readonly ProjectConnection[], connection: Connection | undefined): ConnectionTrunk {
  const first = bindings[0]!
  const source = connection ?? first
  return {
    connectionId: first.connectionId,
    kind: source.kind,
    title: source.title,
    endpoint: endpointOf(source),
    hasCredential: !!source.credentialSetAt,
    bindings
  }
}

/** The project's bindings grouped under their connection, connections in the order their first binding is listed. */
export function connectionTrunks(
  bindings: readonly ProjectConnection[],
  connections: readonly Connection[]
): ConnectionTrunk[] {
  const grouped = new Map<ConnectionId, ProjectConnection[]>()
  for (const binding of bindings) grouped.set(binding.connectionId, [...(grouped.get(binding.connectionId) ?? []), binding])
  return [...grouped].map(([id, members]) => trunkOf(members, connections.find((connection) => connection.id === id)))
}

/** The trunks by kind, clusters first, then GitLab, then Jira; each kind keeps its trunks' order and an empty kind is left out. */
export function trunksByKind(trunks: readonly ConnectionTrunk[]): ConnectionKindGroup[] {
  return KIND_ORDER.map((kind) => ({ kind, trunks: trunks.filter((trunk) => trunk.kind === kind) })).filter(
    (group) => group.trunks.length > 0
  )
}

/** The company's connections this project binds nowhere. */
export function unusedConnections(
  bindings: readonly ProjectConnection[],
  connections: readonly Connection[]
): Connection[] {
  const used = new Set(bindings.map((binding) => binding.connectionId))
  return connections.filter((connection) => !used.has(connection.id))
}

/**
 * The trunk narrowed to a query, ignoring case: whole when its title or endpoint holds it, else
 * only the bindings whose label or target does, and `null` when none does.
 */
export function trunkMatching(trunk: ConnectionTrunk, query: string): ConnectionTrunk | null {
  const wanted = query.trim().toLowerCase()
  const holds = (text: string | null): boolean => !!text && text.toLowerCase().includes(wanted)
  if (wanted === '' || holds(trunk.title) || holds(trunk.endpoint)) return trunk
  const bindings = trunk.bindings.filter((binding) => holds(binding.label) || holds(binding.target))
  return bindings.length ? { ...trunk, bindings } : null
}
