import { compareProjects } from '@/features/catalog/lib/project-group-order'
import { newestFirst } from '@/features/catalog/model/task-code'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'
import type { WorkCounts, WorkGroup, WorkRow, WorkSort, WorkView } from '@/features/work/model/work-view'

const VIEW_RULES: Readonly<Record<WorkView, (row: WorkRow) => boolean>> = {
  needs: (row) => row.needs > 0,
  running: (row) => row.running,
  open: (row) => row.kind !== 'filed',
  filed: (row) => row.kind === 'filed'
}

export function rowsOfView(rows: readonly WorkRow[], view: WorkView): WorkRow[] {
  return rows.filter(VIEW_RULES[view])
}

/** Every word of the query has to appear in the title or the code, whatever the case. */
export function matchesQuery(row: WorkRow, query: string): boolean {
  const words = query.toLowerCase().split(/\s+/).filter((word) => word.length > 0)
  const haystack = `${row.task.title} ${row.task.code}`.toLowerCase()
  return words.every((word) => haystack.includes(word))
}

/**
 * What each tab counts. Needs me counts what the review queue counts (claimed steps plus tasks
 * handed in whole), so it equals the header's review segment when nothing narrows the rows.
 */
export function viewCounts(rows: readonly WorkRow[]): WorkCounts {
  return {
    needs: rows.reduce((sum, row) => sum + row.needs, 0),
    running: rowsOfView(rows, 'running').length,
    open: rowsOfView(rows, 'open').length,
    filed: rowsOfView(rows, 'filed').length
  }
}

const SORTS: Readonly<Record<WorkSort, (a: WorkRow, b: WorkRow) => number>> = {
  task: (a, b) => newestFirst(a.task, b.task),
  needs: (a, b) => b.needs - a.needs,
  tokens: (a, b) => b.tokens - a.tokens,
  updated: (a, b) => b.task.updatedAt.localeCompare(a.task.updatedAt)
}

/** A stable sort: rows the key does not tell apart stay newest first. */
export function sortRows(rows: readonly WorkRow[], sort: WorkSort): WorkRow[] {
  return [...rows].sort((a, b) => SORTS[sort](a, b) || newestFirst(a.task, b.task))
}

function groupOf(shown: readonly WorkRow[], folded: readonly WorkRow[], project: Project | undefined): WorkGroup {
  const first = shown[0]!.task
  return {
    projectId: first.projectId,
    projectTitle: first.projectTitle,
    companyName: first.companyName,
    status: project?.status ?? 'ACTIVE',
    icon: project?.icon ?? 'folder',
    iconImageVersion: project?.iconImageVersion ?? null,
    rows: shown,
    filed: folded,
    needs: shown.reduce((sum, row) => sum + row.needs, 0),
    tokens: [...shown, ...folded].reduce((sum, row) => sum + row.tokens, 0)
  }
}

function bucketsByProject(rows: readonly WorkRow[]): Map<ProjectId, WorkRow[]> {
  const buckets = new Map<ProjectId, WorkRow[]>()
  for (const row of rows) {
    const bucket = buckets.get(row.task.projectId)
    if (bucket) bucket.push(row)
    else buckets.set(row.task.projectId, [row])
  }
  return buckets
}

/**
 * The shown rows gathered under their projects in the navigator's project order. A project with
 * nothing shown has no group. `finished` are the project's filed rows, folded into one line under
 * its group; pass none when the view already lists them.
 */
export function groupByProject(
  shown: readonly WorkRow[],
  finished: readonly WorkRow[],
  projects: readonly Project[],
  acrossCompanies: boolean
): WorkGroup[] {
  const projectById = new Map(projects.map((project) => [project.id, project]))
  const finishedByProject = bucketsByProject(finished)
  return [...bucketsByProject(shown).entries()]
    .map(([projectId, rows]) => groupOf(rows, finishedByProject.get(projectId) ?? [], projectById.get(projectId)))
    .sort(compareProjects(acrossCompanies))
}
