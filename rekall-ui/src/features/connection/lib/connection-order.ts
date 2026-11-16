import type { ConnectionKind } from '@/features/connection/model/connection'

/** Kubernetes first, then GitLab, GitHub, Jira, Confluence, then Grafana: the order every list, picker and filter of connections follows. */
export const KIND_ORDER: readonly ConnectionKind[] = ['KUBERNETES', 'GITLAB', 'GITHUB', 'JIRA', 'CONFLUENCE', 'GRAFANA']

/** The connections by kind, then by label within a kind. */
export function sortedByKind<T extends { readonly kind: ConnectionKind; readonly label: string }>(
  connections: readonly T[]
): T[] {
  return [...connections].sort(
    (a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind) || a.label.localeCompare(b.label)
  )
}
