import { apiClient, request } from './client'
import {
  EnvironmentListSchema,
  EnvironmentSchema,
  ProjectListSchema,
  ProjectSchema,
  TaskListSchema,
  TaskSchema
} from './schemas/catalog.schema'
import type { Environment, Project, ProjectStatus, Task, TaskStatus } from '@/model/catalog'
import type { EnvironmentId, ProjectId, TaskId } from '@/model/branded'

export interface ProjectInput {
  name: string
  status: ProjectStatus
  description: string | null
}

export interface TaskInput {
  name: string
  status: TaskStatus
  description: string | null
  projectId: ProjectId
  environmentId: EnvironmentId | null
}

export interface EnvironmentInput {
  label: string
  namespace: string | null
  kubeconfigPath: string | null
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

export async function fetchEnvironments(): Promise<Environment[]> {
  return request(async () => EnvironmentListSchema.parse(await apiClient('/api/environments')))
}

export async function createEnvironment(input: EnvironmentInput): Promise<Environment> {
  return request(async () =>
    EnvironmentSchema.parse(await apiClient('/api/environments', { method: 'POST', body: input }))
  )
}

export async function updateEnvironment(
  id: EnvironmentId,
  input: EnvironmentInput
): Promise<Environment> {
  return request(async () =>
    EnvironmentSchema.parse(
      await apiClient(`/api/environments/${id}`, { method: 'PUT', body: input })
    )
  )
}

export async function deleteEnvironment(id: EnvironmentId): Promise<void> {
  await request(() => apiClient(`/api/environments/${id}`, { method: 'DELETE' }))
}
