import type { ProjectId, TaskId } from '@/model/branded'
import type { Company, Project, Task } from '@/features/catalog/model/catalog'

/** A place inside a project: one of its tasks, or its task list when `taskId` is null. */
export interface OpenPlace {
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
}

export interface OpenTask {
  readonly taskId: TaskId
  readonly projectId: ProjectId
}

/** The open tasks in the order they were opened, and the place last seen in each project that has one. */
export interface WorkingSet {
  readonly tasks: readonly OpenTask[]
  readonly places: readonly OpenPlace[]
}

export const EMPTY_WORKING_SET: WorkingSet = { tasks: [], places: [] }

export interface ProjectGroup {
  readonly project: Project
  readonly tasks: readonly Task[]
}

export interface CompanyGroup {
  readonly company: Pick<Company, 'id' | 'name'>
  readonly projects: readonly ProjectGroup[]
}
