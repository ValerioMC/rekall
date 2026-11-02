import type { Connection } from '@/features/connection/model/connection'

/** The sentence a screen reader hears when the list changed under the reader: the connection by name, or a general one. */
export function changeAnnouncement(before: readonly Connection[], after: readonly Connection[]): string {
  const previous = new Map(before.map((connection) => [connection.id, connection.updatedAt]))
  const changed = after.find((connection) => previous.get(connection.id) !== connection.updatedAt)
  if (changed) return `Connection ${changed.title} updated`
  return before.length > after.length ? 'A connection was removed' : 'Connections updated'
}
