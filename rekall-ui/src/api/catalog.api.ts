import { apiClient, request } from './client'
import {
  CompanyListSchema,
  CompanySchema,
  ProjectListSchema,
  ProjectSchema,
  TaskListSchema,
  TaskSchema
} from './schemas/catalog.schema'
import type { Company, Project, ProjectStatus, Task, TaskStatus } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

export interface CompanyInput {
  name: string
  description: string | null
}

/**
 * What a write carries. The label is sent as typed and comes back normalised, so the response
 * and not the request is what the store keeps.
 */
export interface ProjectInput {
  label: string
  title: string
  status: ProjectStatus
  description: string | null
  blueprintMarkdown: string | null
  repoFolder: string | null
  companyId: CompanyId
}

export interface TaskInput {
  label: string
  title: string
  status: TaskStatus
  description: string | null
  projectId: ProjectId
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

export async function fetchProjects(): Promise<Project[]> {
  return request(async () => ProjectListSchema.parse(await apiClient('/api/projects')))
}

export async function fetchProject(id: ProjectId): Promise<Project> {
  return request(async () => ProjectSchema.parse(await apiClient(`/api/projects/${id}`)))
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
