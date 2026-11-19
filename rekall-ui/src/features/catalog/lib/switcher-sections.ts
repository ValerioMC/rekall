import { groupProjectsByCompany, matchesProjectQuery, type CompanyProjects } from '@/features/catalog/lib/project-picker'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

export const RECENT_SHOWN = 3

export interface SwitcherSections {
  readonly pinned: readonly Project[]
  readonly recent: readonly Project[]
  readonly companies: readonly CompanyProjects[]
}

function isListed(project: Project, currentId: ProjectId | null): boolean {
  return project.status !== 'ARCHIVED' || project.id === currentId
}

function projectsOf(listed: readonly Project[], ids: readonly ProjectId[]): Project[] {
  return ids
    .map((id) => listed.find((project) => project.id === id))
    .filter((project): project is Project => project !== undefined)
}

/**
 * What the project switcher lists: the pinned projects, the recently opened ones that are not
 * pinned, then every company with its projects. An archived project is left out unless it is the
 * one open, and a search drops "Pinned" and "Recent" so a match is not shown twice.
 */
export function switcherSections(
  projects: readonly Project[],
  recentIds: readonly ProjectId[],
  query: string,
  currentId: ProjectId | null,
  pinnedIds: readonly ProjectId[]
): SwitcherSections {
  const listed = projects.filter((project) => isListed(project, currentId))
  const searching = query.trim() !== ''
  const pinned = searching ? [] : projectsOf(listed, pinnedIds)
  const unpinnedRecentIds = recentIds.filter((id) => !pinnedIds.includes(id))
  return {
    pinned,
    recent: searching ? [] : projectsOf(listed, unpinnedRecentIds).slice(0, RECENT_SHOWN),
    companies: groupProjectsByCompany(listed.filter((project) => matchesProjectQuery(project, query)))
  }
}
