import type { ConnectionDetail, ConnectionProject } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/** What taking a connection away from projects would cost. */
export interface ConnectionImpact {
  readonly companies: number
  readonly projects: number
  readonly bindings: number
}

export function impactOf(detail: ConnectionDetail): ConnectionImpact {
  return {
    companies: detail.companies.length,
    projects: new Set(detail.projects.map((project) => project.projectId)).size,
    bindings: detail.projects.length
  }
}

/** The bindings of a company's projects: the reason the company cannot stop sharing the connection. */
export function bindingsOfCompany(detail: ConnectionDetail, companyId: CompanyId): ConnectionProject[] {
  return detail.projects.filter((project) => project.companyId === companyId)
}

/** The distinct projects of those bindings, in first-seen order. */
export function projectsOf(bindings: readonly ConnectionProject[]): ConnectionProject[] {
  const seen = new Set<string>()
  return bindings.filter((binding) => !seen.has(binding.projectId) && seen.add(binding.projectId))
}

export function plural(count: number, noun: string, many: string = `${noun}s`): string {
  return `${count} ${count === 1 ? noun : many}`
}

/** The projects of a company that can still take a connection, archived ones left out, by title. */
export function bindableProjects<T extends { readonly companyId: CompanyId; readonly status: string; readonly title: string }>(
  projects: readonly T[],
  companyId: CompanyId
): T[] {
  return projects
    .filter((project) => project.companyId === companyId && project.status !== 'ARCHIVED')
    .sort((a, b) => a.title.localeCompare(b.title))
}
