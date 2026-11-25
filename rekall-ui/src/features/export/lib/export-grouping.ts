import type { ProjectExportEntry } from '@/features/export/model/project-export'
import type { CompanyId, ProjectId } from '@/model/branded'

export interface ExportProjectGroup {
  readonly projectId: ProjectId
  readonly projectLabel: string
  readonly projectTitle: string
  readonly entries: readonly ProjectExportEntry[]
  readonly failedCount: number
}

export interface ExportCompanyGroup {
  readonly companyId: CompanyId
  readonly companyName: string
  readonly projects: readonly ExportProjectGroup[]
  readonly count: number
  readonly failedCount: number
}

function heldBy(entries: readonly ProjectExportEntry[], key: (entry: ProjectExportEntry) => string): ProjectExportEntry[][] {
  const groups = new Map<string, ProjectExportEntry[]>()
  for (const entry of entries) groups.set(key(entry), [...(groups.get(key(entry)) ?? []), entry])
  return [...groups.values()]
}

function newestFirst(left: ProjectExportEntry, right: ProjectExportEntry): number {
  return Date.parse(right.createdAt) - Date.parse(left.createdAt)
}

function newestOf(entries: readonly ProjectExportEntry[]): number {
  return Math.max(...entries.map((entry) => Date.parse(entry.createdAt)))
}

function failedIn(entries: readonly ProjectExportEntry[]): number {
  return entries.filter((entry) => entry.status === 'FAILED').length
}

function projectGroupsOf(entries: readonly ProjectExportEntry[]): ExportProjectGroup[] {
  return heldBy(entries, (entry) => entry.projectId)
    .map((held) => {
      const sorted = [...held].sort(newestFirst)
      const first = sorted[0] as ProjectExportEntry
      return { projectId: first.projectId, projectLabel: first.projectLabel, projectTitle: first.projectTitle, entries: sorted, failedCount: failedIn(sorted) }
    })
    .sort((left, right) => newestOf(right.entries) - newestOf(left.entries))
}

/** Every export under its company, then its project; the group holding the newest export first at each level, rows newest first. */
export function groupExports(entries: readonly ProjectExportEntry[]): ExportCompanyGroup[] {
  return heldBy(entries, (entry) => entry.companyId)
    .map((held) => {
      const first = held[0] as ProjectExportEntry
      return { companyId: first.companyId, companyName: first.companyName, projects: projectGroupsOf(held), count: held.length, failedCount: failedIn(held) }
    })
    .sort((left, right) => newestOf(entriesOf(right)) - newestOf(entriesOf(left)))
}

function entriesOf(company: ExportCompanyGroup): ProjectExportEntry[] {
  return company.projects.flatMap((project) => project.entries)
}
