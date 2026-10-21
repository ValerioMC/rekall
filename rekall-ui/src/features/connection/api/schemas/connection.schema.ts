import { z } from 'zod'
import { asCompanyId, asConnectionId, asProjectId, asTaskId } from '@/model/branded'

const KindSchema = z.enum(['GITLAB', 'KUBERNETES'])

export const ConnectionSchema = z.object({
  id: z.string().uuid().transform(asConnectionId),
  companyId: z.string().uuid().transform(asCompanyId),
  kind: KindSchema,
  label: z.string(),
  title: z.string(),
  baseUrl: z.string().nullable(),
  kubeContext: z.string().nullable(),
  credentialSetAt: z.string().nullable(),
  projectCount: z.number().int(),
  updatedAt: z.string()
})

export const ConnectionListSchema = z.array(ConnectionSchema)

export const ConnectionDetailSchema = ConnectionSchema.extend({
  projects: z.array(
    z.object({
      projectId: z.string().uuid().transform(asProjectId),
      label: z.string(),
      title: z.string(),
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
    taskLabel: z.string().nullable(),
    taskTitle: z.string().nullable()
  })
)

export const ConnectionRemovedSchema = z.object({ unboundProjects: z.number().int() })

export const ConnectionStreamEventSchema = z.object({ companyId: z.string().uuid().transform(asCompanyId) })

export const ProjectConnectionSchema = z.object({
  connectionId: z.string().uuid().transform(asConnectionId),
  projectId: z.string().uuid().transform(asProjectId),
  kind: KindSchema,
  label: z.string(),
  title: z.string(),
  baseUrl: z.string().nullable(),
  kubeContext: z.string().nullable(),
  target: z.string().nullable(),
  credentialSetAt: z.string().nullable()
})

export const ProjectConnectionListSchema = z.array(ProjectConnectionSchema)
