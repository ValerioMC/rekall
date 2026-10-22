import { apiClient, request } from '../../../api/client'
import {
  ConnectionAccessListSchema,
  ConnectionCheckSchema,
  ConnectionDetailSchema,
  ConnectionListSchema,
  ConnectionRemovedSchema,
  ConnectionSchema,
  JiraIssueListSchema,
  KubeContextListSchema,
  ProjectConnectionListSchema,
  ProjectConnectionSchema
} from './schemas/connection.schema'
import type {
  Connection,
  ConnectionAccess,
  ConnectionCheck,
  ConnectionDetail,
  ConnectionInput,
  ConnectionRemoved,
  CredentialDraft,
  JiraIssue,
  KubeContextOption,
  ProjectConnection
} from '@/features/connection/model/connection'
import { TaskListSchema } from '@/features/catalog/api/schemas/catalog.schema'
import type { Task } from '@/features/catalog/model/catalog'
import type { CompanyId, ConnectionId, ProjectId } from '@/model/branded'

export async function fetchCompanyConnections(companyId: CompanyId): Promise<Connection[]> {
  return request(async () =>
    ConnectionListSchema.parse(await apiClient(`/api/companies/${companyId}/connections`))
  )
}

export async function fetchConnection(id: ConnectionId): Promise<ConnectionDetail> {
  return request(async () => ConnectionDetailSchema.parse(await apiClient(`/api/connections/${id}`)))
}

export async function createConnection(companyId: CompanyId, input: ConnectionInput): Promise<Connection> {
  return request(async () =>
    ConnectionSchema.parse(
      await apiClient(`/api/companies/${companyId}/connections`, { method: 'POST', body: input })
    )
  )
}

export async function updateConnection(id: ConnectionId, input: ConnectionInput): Promise<Connection> {
  return request(async () =>
    ConnectionSchema.parse(await apiClient(`/api/connections/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteConnection(id: ConnectionId): Promise<ConnectionRemoved> {
  return request(async () =>
    ConnectionRemovedSchema.parse(await apiClient(`/api/connections/${id}`, { method: 'DELETE' }))
  )
}

/** Sends the credential once; the answer only says when it was stored. */
export async function setCredential(id: ConnectionId, draft: CredentialDraft): Promise<Connection> {
  return request(async () =>
    ConnectionSchema.parse(
      await apiClient(`/api/connections/${id}/credential`, { method: 'PUT', body: credentialBody(draft), retry: 0 })
    )
  )
}

function credentialBody(draft: CredentialDraft): Record<string, string | null> {
  switch (draft.kind) {
    case 'GITLAB':
      return { token: draft.token }
    case 'KUBERNETES':
      return { kubeconfig: draft.kubeconfig, context: draft.context }
    case 'JIRA':
      return { email: draft.email.trim(), token: draft.token }
  }
}

export async function clearCredential(id: ConnectionId): Promise<Connection> {
  return request(async () =>
    ConnectionSchema.parse(await apiClient(`/api/connections/${id}/credential`, { method: 'DELETE' }))
  )
}

/** The contexts of a kubeconfig being imported; the server stores nothing. */
export async function previewKubeconfig(kubeconfig: string): Promise<KubeContextOption[]> {
  return request(async () =>
    KubeContextListSchema.parse(
      await apiClient('/api/connections/kubeconfig/contexts', {
        method: 'POST',
        body: { kubeconfig },
        retry: 0
      })
    )
  )
}

export async function checkConnection(id: ConnectionId): Promise<ConnectionCheck> {
  return request(async () =>
    ConnectionCheckSchema.parse(await apiClient(`/api/connections/${id}/check`, { method: 'POST', retry: 0 }))
  )
}

export async function fetchConnectionAccess(
  id: ConnectionId,
  offset: number,
  limit: number
): Promise<ConnectionAccess[]> {
  return request(async () =>
    ConnectionAccessListSchema.parse(
      await apiClient(`/api/connections/${id}/access`, { query: { offset, limit } })
    )
  )
}

export async function fetchProjectConnections(projectId: ProjectId): Promise<ProjectConnection[]> {
  return request(async () =>
    ProjectConnectionListSchema.parse(await apiClient(`/api/projects/${projectId}/connections`))
  )
}

/** Binds the connection to the project, or sets the target of a binding it already has. */
export async function bindConnection(
  projectId: ProjectId,
  connectionId: ConnectionId,
  target: string | null
): Promise<ProjectConnection> {
  return request(async () =>
    ProjectConnectionSchema.parse(
      await apiClient(`/api/projects/${projectId}/connections/${connectionId}`, {
        method: 'PUT',
        body: { target }
      })
    )
  )
}

export async function unbindConnection(projectId: ProjectId, connectionId: ConnectionId): Promise<void> {
  await request(() =>
    apiClient(`/api/projects/${projectId}/connections/${connectionId}`, { method: 'DELETE' })
  )
}

/** The open issues of the Jira project the binding names, the most recently updated first. */
export async function fetchJiraIssues(projectId: ProjectId, connectionId: ConnectionId): Promise<JiraIssue[]> {
  return request(async () =>
    JiraIssueListSchema.parse(
      await apiClient(`/api/projects/${projectId}/connections/${connectionId}/jira-issues`, { retry: 0 })
    )
  )
}

/** Imports the chosen issues as the project's tasks; the ones already imported are passed over. */
export async function importJiraIssues(
  projectId: ProjectId,
  connectionId: ConnectionId,
  keys: readonly string[]
): Promise<Task[]> {
  return request(async () =>
    TaskListSchema.parse(
      await apiClient(`/api/projects/${projectId}/connections/${connectionId}/jira-issues/import`, {
        method: 'POST',
        body: { keys },
        retry: 0
      })
    )
  )
}
