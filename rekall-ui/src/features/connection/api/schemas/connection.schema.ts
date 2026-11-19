import { z } from 'zod'
import { asCompanyId, asConnectionId, asProjectConnectionId, asProjectId, asTaskId, asTerminalId } from '@/model/branded'

export const ConnectionKindSchema = z.enum(['GITLAB', 'GITHUB', 'KUBERNETES', 'JIRA', 'CONFLUENCE', 'GRAFANA'])

export const ConnectionSchema = z.object({
  id: z.string().uuid().transform(asConnectionId),
  companyIds: z.array(z.string().uuid().transform(asCompanyId)),
  kind: ConnectionKindSchema,
  label: z.string(),
  title: z.string(),
  baseUrl: z.string().nullable(),
  kubeContext: z.string().nullable(),
  proxyUrl: z.string().nullable(),
  acceptSelfSigned: z.boolean(),
  rootPath: z.string().nullable(),
  credentialSetAt: z.string().nullable(),
  projectCount: z.number().int(),
  updatedAt: z.string()
})

export const ConnectionListSchema = z.array(ConnectionSchema)

export const ConnectionDetailSchema = ConnectionSchema.extend({
  companies: z.array(z.object({ id: z.string().uuid().transform(asCompanyId), name: z.string() })),
  projects: z.array(
    z.object({
      projectId: z.string().uuid().transform(asProjectId),
      companyId: z.string().uuid().transform(asCompanyId),
      label: z.string(),
      title: z.string(),
      bindingLabel: z.string(),
      target: z.string().nullable()
    })
  )
})

export const KubeContextListSchema = z.array(
  z.object({
    name: z.string(),
    server: z.string().nullable(),
    namespace: z.string().nullable(),
    userKind: z.enum(['token', 'client-certificate', 'exec', 'auth-provider', 'basic-auth', 'nothing']),
    supported: z.boolean(),
    refusal: z.string().nullable()
  })
)

export const RevealedCredentialSchema = z.object({
  token: z.string().nullable(),
  kubeconfig: z.string().nullable(),
  email: z.string().nullable(),
  user: z.string().nullable()
})

export const ConnectionCheckSchema = z.object({
  ok: z.boolean(),
  detail: z.string(),
  checkedAt: z.string()
})

export const ConnectionAccessListSchema = z.array(
  z.object({
    id: z.string(),
    accessedAt: z.string(),
    granted: z.boolean(),
    reason: z.string().nullable(),
    terminalId: z.string(),
    taskId: z.string().uuid().transform(asTaskId).nullable(),
    taskCode: z.string().nullable(),
    taskTitle: z.string().nullable()
  })
)

export const JiraIssueListSchema = z.array(
  z.object({
    key: z.string(),
    summary: z.string(),
    status: z.string().nullable(),
    issueType: z.string().nullable(),
    url: z.string(),
    taskId: z.string().uuid().transform(asTaskId).nullable()
  })
)

export const TargetSuggestionListSchema = z.array(z.object({ value: z.string(), name: z.string() }))

export const ConfluenceImportSchema = z.object({
  spaceKey: z.string(),
  spaceName: z.string(),
  pagesRead: z.number().int(),
  descriptionWritten: z.boolean(),
  blueprintWritten: z.boolean(),
  sections: z.array(z.object({ title: z.string(), pages: z.number().int(), outcome: z.enum(['created', 'replaced', 'kept']) }))
})

export const ConnectionRemovedSchema = z.object({ unboundProjects: z.number().int() })

export const ConnectionStreamEventSchema = z.object({ companyIds: z.array(z.string().uuid().transform(asCompanyId)) })

export const ProjectConnectionSchema = z.object({
  id: z.string().uuid().transform(asProjectConnectionId),
  connectionId: z.string().uuid().transform(asConnectionId),
  projectId: z.string().uuid().transform(asProjectId),
  kind: ConnectionKindSchema,
  label: z.string(),
  title: z.string(),
  baseUrl: z.string().nullable(),
  kubeContext: z.string().nullable(),
  target: z.string().nullable(),
  taskManager: z.boolean(),
  credentialSetAt: z.string().nullable()
})

export const ProjectConnectionListSchema = z.array(ProjectConnectionSchema)

export const RepositoryDiscoverySchema = z.object({
  folder: z.string(),
  repositories: z.array(
    z.object({
      folder: z.string(),
      origin: z.string().nullable(),
      outcome: z.enum(['BOUND', 'ALREADY_BOUND', 'NO_ORIGIN', 'NO_CONNECTION', 'NO_CREDENTIAL']),
      connectionLabel: z.string().nullable(),
      bindingId: z.string().uuid().transform(asProjectConnectionId).nullable(),
      label: z.string().nullable(),
      target: z.string().nullable(),
      component: z.string().nullable()
    })
  )
})

export const CredentialAccessEventSchema = z.object({
  terminalId: z.string().uuid().transform(asTerminalId),
  connectionId: z.string().uuid().transform(asConnectionId),
  granted: z.boolean()
})
