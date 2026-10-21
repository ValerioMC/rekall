import type { Connection } from '@/features/connection/model/connection'

/** The host a URL names, or the text unchanged when it does not parse, so nothing typed is hidden. */
export function hostOf(url: string): string {
  try {
    return new URL(url).host || url
  } catch {
    return url
  }
}

/** Where a connection points, in the words a row shows: the instance host, or the context and its server. */
export function endpointOf(connection: Pick<Connection, 'kind' | 'baseUrl' | 'kubeContext'>): string {
  const host = connection.baseUrl ? hostOf(connection.baseUrl) : null
  if (connection.kind === 'GITLAB') return host ?? 'no instance URL'
  return (
    [connection.kubeContext, host].filter((part): part is string => !!part).join(' · ') || 'no kubeconfig yet'
  )
}
