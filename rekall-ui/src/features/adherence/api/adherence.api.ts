import { apiClient, request } from '../../../api/client'
import { PlanAdherenceSchema, ProjectAdherenceSchema } from './schemas/adherence.schema'
import type { PlanAdherence, ProjectAdherence } from '@/features/adherence/model/adherence'
import type { ProjectId, TaskId } from '@/model/branded'

export async function fetchTaskAdherence(taskId: TaskId): Promise<PlanAdherence> {
  return request(async () => PlanAdherenceSchema.parse(await apiClient(`/api/tasks/${taskId}/adherence`)))
}

export async function fetchProjectAdherence(projectId: ProjectId): Promise<ProjectAdherence> {
  return request(async () => ProjectAdherenceSchema.parse(await apiClient(`/api/projects/${projectId}/adherence`)))
}
