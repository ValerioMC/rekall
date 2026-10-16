import { partitionTasks, type PartitionedTasks } from '@/features/catalog/lib/partition-tasks'
import { compareProjects } from '@/features/catalog/lib/project-group-order'
import type { Project, ProjectStatus, Task } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

/** The navigator's card for one project: what it is, and its tasks split into live, parked and filed. */
export interface ProjectTaskGroup extends PartitionedTasks {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly status: ProjectStatus
  readonly icon: string
  readonly iconImageVersion: string | null
  readonly tasks: readonly Task[]
}

/**
 * The visible tasks gathered under their projects, in the navigator's project order. A project the
 * catalog no longer lists still gets a card, drawn as active with the default icon.
 */
export function groupTasksByProject(
  tasks: readonly Task[],
  projects: readonly Project[],
  acrossCompanies: boolean
): ProjectTaskGroup[] {
  const projectById = new Map(projects.map((project) => [project.id, project]))
  const byProject = new Map<ProjectId, Task[]>()
  for (const task of tasks) {
    const bucket = byProject.get(task.projectId)
    if (bucket) bucket.push(task)
    else byProject.set(task.projectId, [task])
  }
  return [...byProject.entries()]
    .map(([projectId, projectTasks]): ProjectTaskGroup => {
      const project = projectById.get(projectId)
      const first = projectTasks[0]!
      return {
        projectId,
        projectTitle: first.projectTitle,
        companyName: first.companyName,
        status: project?.status ?? 'ACTIVE',
        icon: project?.icon ?? 'folder',
        iconImageVersion: project?.iconImageVersion ?? null,
        tasks: projectTasks,
        ...partitionTasks(projectTasks)
      }
    })
    .sort(compareProjects(acrossCompanies))
}
