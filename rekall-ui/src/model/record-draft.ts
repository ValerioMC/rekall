import type { CompanyId, ProjectId, TaskId } from './branded'
import type { Company, Project, ProjectStatus, Task, TaskStatus } from './catalog'

/**
 * What the record editor is working on, for any of the three levels.
 *
 * A discriminated union rather than one optional-everything shape: the editor renders a status
 * field for a project and a task and not for a company, and a type that can express "a company
 * with a project status" would make that a runtime question.
 *
 * A null id means the record does not exist yet. The parent id is always present, because a
 * project without a company and a task without a project are not things this application can
 * hold.
 */
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
    projectId: task?.projectId ?? projectId
  }
}
