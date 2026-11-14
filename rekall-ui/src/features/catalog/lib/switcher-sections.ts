import { groupProjectsByCompany, matchesProjectQuery, type CompanyProjects } from '@/features/catalog/lib/project-picker'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

export const RECENT_SHOWN = 3

export interface SwitcherSections {
  readonly recent: readonly Project[]
  readonly companies: readonly CompanyProjects[]
}

function isListed(project: Project, currentId: ProjectId | null): boolean {
  return project.status !== 'ARCHIVED' || project.id === currentId
}

function recentProjects(listed: readonly Project[], recentIds: readonly ProjectId[]): Project[] {
  return recentIds
    .map((id) => listed.find((project) => project.id === id))
    .filter((project): project is Project => project !== undefined)
    .slice(0, RECENT_SHOWN)
}

/**
 * What the project switcher lists: the recently opened projects first, then every company with its
 * projects. An archived project is left out unless it is the one open, and a search drops "Recent"
 * so the match is not shown twice.
 */
export function switcherSections(
  projects: readonly Project[],
  recentIds: readonly ProjectId[],
  query: string,
  currentId: ProjectId | null
): SwitcherSections {
  const listed = projects.filter((project) => isListed(project, currentId))
  const searching = query.trim() !== ''
  return {
    recent: searching ? [] : recentProjects(listed, recentIds),
    companies: groupProjectsByCompany(listed.filter((project) => matchesProjectQuery(project, query)))
  }
}
