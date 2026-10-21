import { apiClient, request } from '../../../api/client'
import {
  ConnectionAccessListSchema,
  ConnectionCheckSchema,
  ConnectionDetailSchema,
  ConnectionListSchema,
  ConnectionRemovedSchema,
  ConnectionSchema,
  KubeContextListSchema
} from './schemas/connection.schema'
import type {
  Connection,
  ConnectionAccess,
  ConnectionCheck,
  ConnectionDetail,
  ConnectionInput,
  ConnectionRemoved,
  CredentialDraft,
  KubeContextOption
} from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

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
  const body =
    draft.kind === 'GITLAB'
      ? { token: draft.token }
      : { kubeconfig: draft.kubeconfig, context: draft.context }
  return request(async () =>
    ConnectionSchema.parse(
      await apiClient(`/api/connections/${id}/credential`, { method: 'PUT', body, retry: 0 })
    )
  )
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
