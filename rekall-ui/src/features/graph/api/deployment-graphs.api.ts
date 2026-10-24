import { apiClient, request } from '../../../api/client'
import { TaskStepListSchema } from '../../catalog/api/schemas/catalog.schema'
import { DeploymentGraphListSchema, DeploymentGraphSchema } from './schemas/deployment-graph.schema'
import type { DeploymentGraph, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { TaskStep } from '@/features/catalog/model/catalog'
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

/** Lays the graph on a task with no steps; the answer is the task's new checklist. */
export async function attachDeploymentGraph(taskId: TaskId, graphId: DeploymentGraphId): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/tasks/${taskId}/graph`, { method: 'POST', body: { graphId } }))
  )
}
