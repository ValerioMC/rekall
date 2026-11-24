import type { Company, Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyGroup, OpenTask, ProjectGroup } from '@/features/workingset/model/working-set'

/**
 * The open tasks grouped by company and project, companies by name and projects by title, each
 * project's tasks in the order they were opened. A task or project the catalog does not hold is
 * dropped: the set can name a task the console has not loaded or that was deleted.
 */
export function groupByProject(
  open: readonly OpenTask[],
  companies: readonly Company[],
  projects: readonly Project[],
  tasks: readonly Task[]
): CompanyGroup[] {
  const taskById = new Map(tasks.map((task) => [task.id, task]))
  const groups = projects
    .map((project) => projectGroupOf(project, open, taskById))
    .filter((group) => group.tasks.length > 0)
  return companies
    .map((company) => ({
      company: { id: company.id, name: company.name },
      projects: groups.filter((group) => group.project.companyId === company.id).sort(byProjectTitle)
    }))
    .filter((group) => group.projects.length > 0)
    .sort((left, right) => left.company.name.localeCompare(right.company.name))
}

function projectGroupOf(project: Project, open: readonly OpenTask[], taskById: ReadonlyMap<string, Task>): ProjectGroup {
  const opened = open
    .filter((entry) => entry.projectId === project.id)
    .map((entry) => taskById.get(entry.taskId))
    .filter((task): task is Task => task !== undefined)
  return { project, tasks: opened }
}

function byProjectTitle(left: ProjectGroup, right: ProjectGroup): number {
  return left.project.title.localeCompare(right.project.title)
}
