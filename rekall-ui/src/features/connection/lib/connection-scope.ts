import { endpointOf, hostOf } from '@/features/connection/lib/connection-endpoint'
import type { Connection } from '@/features/connection/model/connection'

/** What a connection covers, in one line: the host and the root path under it, or where it points when it has none. */
export function scopeOf(connection: Pick<Connection, 'kind' | 'baseUrl' | 'kubeContext' | 'rootPath'>): string {
  if (!connection.rootPath || !connection.baseUrl) return endpointOf(connection)
  return `${hostOf(connection.baseUrl)}/${connection.rootPath}`
}
