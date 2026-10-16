import type { CompanyId } from '@/model/branded'
import type { Project } from '@/features/catalog/model/catalog'

export interface CompanyProjects {
  readonly companyId: CompanyId
  readonly companyName: string
  readonly projects: readonly Project[]
}

/** Every whitespace-separated part of the needle must appear in the project's company, title, label or anchor. */
export function matchesProjectQuery(project: Project, needle: string): boolean {
  const hay = `${project.companyName} ${project.title} ${project.label} ${project.anchor}`.toLowerCase()
  return needle
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .every((part) => hay.includes(part))
}

/** Companies by name; inside one, active projects first, then by title. Companies left empty are dropped. */
export function groupProjectsByCompany(projects: readonly Project[]): CompanyProjects[] {
  const byCompany = new Map<CompanyId, Project[]>()
  for (const project of projects) {
    const bucket = byCompany.get(project.companyId) ?? []
    bucket.push(project)
    byCompany.set(project.companyId, bucket)
  }
  return [...byCompany.values()]
    .map((bucket) => ({
      companyId: bucket[0]!.companyId,
      companyName: bucket[0]!.companyName,
      projects: bucket.sort(
        (a, b) =>
          Number(b.status === 'ACTIVE') - Number(a.status === 'ACTIVE') ||
          a.title.localeCompare(b.title)
      )
    }))
    .sort((a, b) => a.companyName.localeCompare(b.companyName))
}
