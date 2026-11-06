import type { Connection, ConnectionDetail, ConnectionKind, ConnectionProject } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId, ProjectId } from '@/model/branded'

export const ACME = 'company-acme' as CompanyId
export const GLOBEX = 'company-globex' as CompanyId

export function aConnection(id: string, over: Partial<Connection> = {}): Connection {
  return {
    id: id as ConnectionId,
    companyIds: [ACME],
    kind: 'GITLAB',
    label: id,
    title: `The ${id}`,
    baseUrl: 'https://gitlab.example.com',
    kubeContext: null,
    proxyUrl: null,
    acceptSelfSigned: false,
    rootPath: null,
    credentialSetAt: '2026-10-21T10:00:00Z',
    projectCount: 0,
    updatedAt: '2026-10-21T10:00:00Z',
    ...over
  }
}

export function aKindedConnection(id: string, kind: ConnectionKind, over: Partial<Connection> = {}): Connection {
  return aConnection(id, { kind, ...over })
}

export function aBinding(projectId: string, companyId: CompanyId, over: Partial<ConnectionProject> = {}): ConnectionProject {
  return {
    projectId: projectId as ProjectId,
    companyId,
    label: projectId,
    title: `Project ${projectId}`,
    bindingLabel: projectId,
    target: null,
    ...over
  }
}

export function aDetail(connection: Connection, projects: readonly ConnectionProject[] = []): ConnectionDetail {
  const names: Record<string, string> = { [ACME]: 'Acme', [GLOBEX]: 'Globex' }
  return {
    ...connection,
    companies: connection.companyIds.map((id) => ({ id, name: names[id] ?? id })),
    projects
  }
}
