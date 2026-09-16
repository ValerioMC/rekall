import type { Task } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

/**
 * How the note-side pickers find a task: every whitespace-separated part of the needle has to
 * appear somewhere in the project's label or title or the task's label or title.
 */
export function matchesTaskQuery(task: Task, needle: string): boolean {
  const hay = `${task.projectLabel} ${task.projectTitle} ${task.label} ${task.title}`.toLowerCase()
  return needle
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .every((part) => hay.includes(part))
}

export interface ProjectGroup<Row> {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly rows: Row[]
}

/** Rows bucketed by the task's project, projects by title, rows in the order they came. */
export function groupByProject<Row>(
  rows: readonly Row[],
  taskOf: (row: Row) => Task
): ProjectGroup<Row>[] {
  const byProject = new Map<ProjectId, ProjectGroup<Row>>()
  for (const row of rows) {
    const task = taskOf(row)
    let group = byProject.get(task.projectId)
    if (!group) {
      group = {
        projectId: task.projectId,
        projectTitle: task.projectTitle,
        companyName: task.companyName,
        rows: []
      }
      byProject.set(task.projectId, group)
    }
    group.rows.push(row)
  }
  return [...byProject.values()].sort((a, b) => a.projectTitle.localeCompare(b.projectTitle))
}
