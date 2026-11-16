import { apiClient, request } from '../../../api/client'
import { ProjectComponentListSchema, ProjectComponentSchema } from './schemas/project-component.schema'
import type { ProjectComponent, ProjectComponentInput } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

export async function fetchProjectComponents(projectId: ProjectId): Promise<ProjectComponent[]> {
  return request(async () =>
    ProjectComponentListSchema.parse(await apiClient(`/api/projects/${projectId}/components`))
  )
}

export async function createProjectComponent(
  projectId: ProjectId,
  input: ProjectComponentInput
): Promise<ProjectComponent> {
  return request(async () =>
    ProjectComponentSchema.parse(
      await apiClient(`/api/projects/${projectId}/components`, { method: 'POST', body: input })
    )
  )
}

export async function updateProjectComponent(
  id: ProjectComponentId,
  input: ProjectComponentInput
): Promise<ProjectComponent> {
  return request(async () =>
    ProjectComponentSchema.parse(
      await apiClient(`/api/project-components/${id}`, { method: 'PUT', body: input })
    )
  )
}

export async function reorderProjectComponents(
  projectId: ProjectId,
  componentIds: readonly ProjectComponentId[]
): Promise<ProjectComponent[]> {
  return request(async () =>
    ProjectComponentListSchema.parse(
      await apiClient(`/api/projects/${projectId}/components/order`, {
        method: 'PUT',
        body: { componentIds }
      })
    )
  )
}

export async function deleteProjectComponent(id: ProjectComponentId): Promise<void> {
  await request(() => apiClient(`/api/project-components/${id}`, { method: 'DELETE' }))
}
