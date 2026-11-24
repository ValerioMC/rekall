import type { Company, Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'
import { task as aTask } from '../project/project-task-fixtures'

export function company(id: string, name: string): Company {
  return { id: id as CompanyId, name, description: null, logoImageVersion: null, projectCount: 0, taskCount: 0, updatedAt: '2026-09-20T10:00:00Z' }
}

export function project(id: string, title: string, companyId: string): Project {
  return {
    id: id as ProjectId,
    label: id,
    taskKey: id.toUpperCase().slice(0, 2),
    title,
    status: 'ACTIVE',
    icon: 'folder',
    iconImageVersion: null,
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    mainBranch: null,
    pullRequestFlow: false,
    companyId: companyId as CompanyId,
    companyName: companyId,
    taskCount: 0,
    anchor: `project:${id}`,
    updatedAt: '2026-09-20T10:00:00Z'
  }
}

export function task(id: string, projectId: string): Task {
  return aTask(id, { projectId: projectId as ProjectId })
}

export function open(taskId: string, projectId: string): { taskId: TaskId; projectId: ProjectId } {
  return { taskId: taskId as TaskId, projectId: projectId as ProjectId }
}
