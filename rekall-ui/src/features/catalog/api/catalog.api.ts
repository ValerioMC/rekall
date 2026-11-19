import { apiClient, request } from '../../../api/client'
import {
  CompanyListSchema,
  CompanySchema,
  ProjectListSchema,
  ProjectRepositorySchema,
  ProjectSchema,
  TagListSchema,
  TagSchema,
  TaskListSchema,
  TaskSchema
} from './schemas/catalog.schema'
import type {
  Company,
  Project,
  ProjectRepository,
  Tag,
  Task,
  TaskKind
} from '@/features/catalog/model/catalog'
import type { ProjectStatus } from '@/features/catalog/model/project-status'
import type { TaskStatus } from '@/features/catalog/model/task-status'
import type { TaskStepState } from '@/features/steps/model/task-step'
import type { CompanyId, ProjectId, TagId, TaskId } from '@/model/branded'

export interface CompanyInput {
  name: string
  description: string | null
}

export interface ProjectInput {
  label: string
  /** Blank or null: the server suggests one from the label. On an update, null keeps the stored one. */
  taskKey: string | null
  title: string
  status: ProjectStatus
  icon: string
  description: string | null
  blueprintMarkdown: string | null
  repoFolder: string | null
  autoCommit: boolean
  mainBranch: string | null
  pullRequestFlow: boolean
  companyId: CompanyId
}

export interface TaskInput {
  title: string
  status: TaskStatus
  description: string | null
  projectId: ProjectId
  tagId: TagId | null
  kind: TaskKind
  epicId: TaskId | null
}

export interface TagInput {
  name: string
  icon: string
  color: string
}

export async function fetchCompanies(): Promise<Company[]> {
  return request(async () => CompanyListSchema.parse(await apiClient('/api/companies')))
}

export async function createCompany(input: CompanyInput): Promise<Company> {
  return request(async () =>
    CompanySchema.parse(await apiClient('/api/companies', { method: 'POST', body: input }))
  )
}

export async function updateCompany(id: CompanyId, input: CompanyInput): Promise<Company> {
  return request(async () =>
    CompanySchema.parse(await apiClient(`/api/companies/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteCompany(id: CompanyId): Promise<void> {
  await request(() => apiClient(`/api/companies/${id}`, { method: 'DELETE' }))
}

/** The body is the image file itself, in any format the server can decode. */
export async function uploadCompanyLogoImage(id: CompanyId, file: File): Promise<Company> {
  return request(async () =>
    CompanySchema.parse(
      await apiClient(`/api/companies/${id}/logo-image`, {
        method: 'PUT',
        body: file,
        headers: { 'Content-Type': file.type || 'application/octet-stream' }
      })
    )
  )
}

export async function removeCompanyLogoImage(id: CompanyId): Promise<void> {
  await request(() => apiClient(`/api/companies/${id}/logo-image`, { method: 'DELETE' }))
}

export async function fetchProjects(): Promise<Project[]> {
  return request(async () => ProjectListSchema.parse(await apiClient('/api/projects')))
}

export async function fetchProject(id: ProjectId): Promise<Project> {
  return request(async () => ProjectSchema.parse(await apiClient(`/api/projects/${id}`)))
}

export async function fetchProjectRepository(id: ProjectId): Promise<ProjectRepository> {
  return request(async () =>
    ProjectRepositorySchema.parse(await apiClient(`/api/projects/${id}/repository`))
  )
}

export async function createProject(input: ProjectInput): Promise<Project> {
  return request(async () =>
    ProjectSchema.parse(await apiClient('/api/projects', { method: 'POST', body: input }))
  )
}

export async function updateProject(id: ProjectId, input: ProjectInput): Promise<Project> {
  return request(async () =>
    ProjectSchema.parse(await apiClient(`/api/projects/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteProject(id: ProjectId): Promise<void> {
  await request(() => apiClient(`/api/projects/${id}`, { method: 'DELETE' }))
}

/** The body is the image file itself, in any format the server can decode. */
export async function uploadProjectIconImage(id: ProjectId, file: File): Promise<Project> {
  return request(async () =>
    ProjectSchema.parse(
      await apiClient(`/api/projects/${id}/icon-image`, {
        method: 'PUT',
        body: file,
        headers: { 'Content-Type': file.type || 'application/octet-stream' }
      })
    )
  )
}

export async function removeProjectIconImage(id: ProjectId): Promise<void> {
  await request(() => apiClient(`/api/projects/${id}/icon-image`, { method: 'DELETE' }))
}

export async function fetchTasks(projectId?: ProjectId): Promise<Task[]> {
  return request(async () =>
    TaskListSchema.parse(await apiClient('/api/tasks', { query: projectId ? { projectId } : {} }))
  )
}

export async function fetchTask(id: TaskId): Promise<Task> {
  return request(async () => TaskSchema.parse(await apiClient(`/api/tasks/${id}`)))
}

export async function createTask(input: TaskInput): Promise<Task> {
  return request(async () =>
    TaskSchema.parse(await apiClient('/api/tasks', { method: 'POST', body: input }))
  )
}

export async function updateTask(id: TaskId, input: TaskInput): Promise<Task> {
  return request(async () =>
    TaskSchema.parse(await apiClient(`/api/tasks/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteTask(id: TaskId): Promise<void> {
  await request(() => apiClient(`/api/tasks/${id}`, { method: 'DELETE' }))
}

/** Writes the task's title and description to its issue in the project's task manager, creating the issue if it has none. */
export async function syncTaskTracker(id: TaskId): Promise<Task> {
  return request(async () => TaskSchema.parse(await apiClient(`/api/tasks/${id}/tracker-sync`, { method: 'POST' })))
}

/** Accept (`DONE`) or send back (`OPEN`) a stepless task's review line; other states are refused. */
export async function reviewTask(id: TaskId, reviewState: TaskStepState): Promise<Task> {
  return request(async () =>
    TaskSchema.parse(
      await apiClient(`/api/tasks/${id}/review`, {
        method: 'PATCH',
        body: { reviewState }
      })
    )
  )
}

export async function fetchTags(): Promise<Tag[]> {
  return request(async () => TagListSchema.parse(await apiClient('/api/tags')))
}

export async function createTag(input: TagInput): Promise<Tag> {
  return request(async () => TagSchema.parse(await apiClient('/api/tags', { method: 'POST', body: input })))
}

export async function updateTag(id: TagId, input: TagInput): Promise<Tag> {
  return request(async () =>
    TagSchema.parse(await apiClient(`/api/tags/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteTag(id: TagId): Promise<void> {
  await request(() => apiClient(`/api/tags/${id}`, { method: 'DELETE' }))
}
