import { CONNECTION_STATUSES } from '@/features/connection/lib/connection-filter'
import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import type { ConnectionFilter } from '@/features/connection/lib/connection-filter'
import { asCompanyId } from '@/model/branded'

export type RouteQuery = Readonly<Record<string, string | readonly (string | null)[] | null | undefined>>

function single(value: RouteQuery[string]): string | null {
  const first = Array.isArray(value) ? value[0] : value
  return typeof first === 'string' && first !== '' ? first : null
}

/** A `returnTo` is followed only when it stays inside the app. */
export function internalPath(value: string | null): string | null {
  return value !== null && value.startsWith('/') && !value.startsWith('//') ? value : null
}

export function filterFromQuery(query: RouteQuery): ConnectionFilter {
  const company = single(query.company)
  const kind = single(query.kind)
  const status = single(query.status)
  return {
    companyId: company ? asCompanyId(company) : null,
    kind: KIND_ORDER.find((known) => known === kind) ?? null,
    status: CONNECTION_STATUSES.find((known) => known === status) ?? null
  }
}

export function returnToFromQuery(query: RouteQuery): string | null {
  return internalPath(single(query.returnTo))
}

/** The query that keeps a filter and a way back; a filter at rest leaves no key. */
export function queryOf(filter: ConnectionFilter, returnTo: string | null): Record<string, string> {
  const entries: [string, string | null][] = [
    ['company', filter.companyId],
    ['kind', filter.kind],
    ['status', filter.status],
    ['returnTo', returnTo]
  ]
  return Object.fromEntries(entries.filter((entry): entry is [string, string] => entry[1] !== null))
}

/** What the way back is called: the project it came from by name, or the screen. */
export function returnLabelOf(path: string, projectTitles: ReadonlyMap<string, string>): string {
  const project = /^\/projects\/([^/?#]+)/.exec(path)
  if (project) return projectTitles.get(project[1]!) ?? 'Project'
  if (path.startsWith('/projects')) return 'Projects'
  if (path.startsWith('/companies')) return 'Companies'
  return 'where you were'
}
