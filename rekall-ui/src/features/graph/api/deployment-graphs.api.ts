import { apiClient, request } from '../../../api/client'
import { z } from 'zod'
import { TaskSchema, TaskStepListSchema } from '../../catalog/api/schemas/catalog.schema'
import { DeploymentGraphListSchema, DeploymentGraphSchema } from './schemas/deployment-graph.schema'
import type { DeploymentGraph, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import type { DeploymentGraphId, ProjectId, TaskId } from '@/model/branded'

export async function fetchDeploymentGraphs(projectId: ProjectId): Promise<DeploymentGraph[]> {
  return request(async () => DeploymentGraphListSchema.parse(await apiClient(`/api/projects/${projectId}/graphs`)))
}

export async function createDeploymentGraph(projectId: ProjectId, draft: GraphDraft): Promise<DeploymentGraph> {
  return request(async () =>
    DeploymentGraphSchema.parse(await apiClient(`/api/projects/${projectId}/graphs`, { method: 'POST', body: draft }))
  )
}

/** Writes the graph whole: title, description and every step and route. */
export async function replaceDeploymentGraph(id: DeploymentGraphId, draft: GraphDraft): Promise<DeploymentGraph> {
  return request(async () =>
    DeploymentGraphSchema.parse(await apiClient(`/api/graphs/${id}`, { method: 'PUT', body: draft }))
  )
}

export async function deleteDeploymentGraph(id: DeploymentGraphId): Promise<void> {
  await request(() => apiClient(`/api/graphs/${id}`, { method: 'DELETE' }))
}

/** Lays the graph after the task's own steps; the answer is the task's new checklist. */
export async function attachDeploymentGraph(taskId: TaskId, graphId: DeploymentGraphId): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/tasks/${taskId}/graph`, { method: 'POST', body: { graphId } }))
  )
}

const ReleaseOnRunSchema = z.object({ laid: z.string().nullable(), steps: TaskStepListSchema })

/** What running the task laid: the release graph's title, or null when nothing was laid, and its checklist. */
export interface ReleaseOnRun {
  readonly laid: string | null
  readonly steps: TaskStep[]
}

/** Makes the graph the one the project releases every task it runs with, or clears it; answers every graph. */
export async function chooseReleaseGraph(projectId: ProjectId, graphId: DeploymentGraphId | null): Promise<DeploymentGraph[]> {
  return request(async () =>
    DeploymentGraphListSchema.parse(
      await apiClient(`/api/projects/${projectId}/release-graph`, { method: 'PUT', body: { graphId } })
    )
  )
}

/** Lays the project's release graph as the console starts a session, unless the task has one or is left unreleased. */
export async function releaseOnRun(taskId: TaskId): Promise<ReleaseOnRun> {
  return request(async () => ReleaseOnRunSchema.parse(await apiClient(`/api/tasks/${taskId}/release`, { method: 'POST' })))
}

export async function skipRelease(taskId: TaskId, skipped: boolean): Promise<Task> {
  return request(async () =>
    TaskSchema.parse(await apiClient(`/api/tasks/${taskId}/release-skipped`, { method: 'PUT', body: { skipped } }))
  )
}
