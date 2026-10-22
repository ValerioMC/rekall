import type { CompanyId, ProjectId, TagId, TaskId } from '../../../model/branded'
import type { Company, Project, ProjectStatus, Task, TaskStatus } from './catalog'

export type RecordDraft =
  | {
      kind: 'company'
      id: CompanyId | null
      name: string
      description: string
    }
  | {
      kind: 'project'
      id: ProjectId | null
      label: string
      title: string
      description: string
      status: ProjectStatus
      icon: string
      companyId: CompanyId
    }
  | {
      kind: 'task'
      id: TaskId | null
      label: string
      title: string
      description: string
      status: TaskStatus
      projectId: ProjectId
      tagId: TagId | null
      /** The skills a new task starts with; an existing task's are changed on its own pane. */
      skills: string[]
    }

export function companyDraft(company?: Company): RecordDraft {
  return {
    kind: 'company',
    id: company?.id ?? null,
    name: company?.name ?? '',
    description: company?.description ?? ''
  }
}

export function projectDraft(companyId: CompanyId, project?: Project): RecordDraft {
  return {
    kind: 'project',
    id: project?.id ?? null,
    label: project?.label ?? '',
    title: project?.title ?? '',
    description: project?.description ?? '',
    status: project?.status ?? 'ACTIVE',
    icon: project?.icon ?? 'folder',
    companyId: project?.companyId ?? companyId
  }
}

export function taskDraft(projectId: ProjectId, task?: Task): RecordDraft {
  return {
    kind: 'task',
    id: task?.id ?? null,
    label: task?.label ?? '',
    title: task?.title ?? '',
    description: task?.description ?? '',
    status: task?.status ?? 'TODO',
    projectId: task?.projectId ?? projectId,
    tagId: task?.tagId ?? null,
    skills: []
  }
}
