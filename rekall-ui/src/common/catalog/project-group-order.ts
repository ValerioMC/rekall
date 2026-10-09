import { compareText } from '@/common/catalog/compare-text'
import type { ProjectStatus } from '@/model/catalog'

export interface OrderedProject {
  readonly projectTitle: string
  readonly companyName: string
  readonly status: ProjectStatus
}

// Active projects float to the top so a paused or done one never buries what is live.
const STATUS_RANK: Readonly<Record<ProjectStatus, number>> = { ACTIVE: 0, PAUSED: 1, DONE: 2, ARCHIVED: 3 }

/**
 * Projects by status, then title. Across companies (`All work`) they are first gathered under
 * their company, so a company's projects sit together instead of interleaving with another's.
 */
export function compareProjects(acrossCompanies: boolean) {
  return (a: OrderedProject, b: OrderedProject): number =>
    (acrossCompanies ? compareText(a.companyName, b.companyName) : 0) ||
    STATUS_RANK[a.status] - STATUS_RANK[b.status] ||
    compareText(a.projectTitle, b.projectTitle)
}
