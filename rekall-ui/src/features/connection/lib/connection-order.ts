import type { ConnectionKind } from '@/features/connection/model/connection'

/** Clusters first, then GitLab, then Jira: the order every list of a project's connections follows. */
export const KIND_ORDER: readonly ConnectionKind[] = ['KUBERNETES', 'GITLAB', 'JIRA']

/** The connections by kind, then by label within a kind. */
export function sortedByKind<T extends { readonly kind: ConnectionKind; readonly label: string }>(
  connections: readonly T[]
): T[] {
  return [...connections].sort(
    (a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind) || a.label.localeCompare(b.label)
  )
}
