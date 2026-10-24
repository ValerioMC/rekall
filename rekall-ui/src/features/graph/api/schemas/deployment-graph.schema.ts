import { z } from 'zod'
import { asConnectionId, asDeploymentGraphId, asGraphStepId, asProjectId } from '@/model/branded'

const graphStepId = z.string().uuid().transform(asGraphStepId)

export const DeploymentGraphSchema = z.object({
  id: z.string().uuid().transform(asDeploymentGraphId),
  projectId: z.string().uuid().transform(asProjectId),
  title: z.string(),
  description: z.string(),
  steps: z.array(
    z.object({
      id: graphStepId,
      title: z.string(),
      detail: z.string().nullable(),
      connectionId: z.string().uuid().transform(asConnectionId).nullable(),
      maxVisits: z.number().int().nullable(),
      routes: z.array(z.object({ outcome: z.string(), toStepId: graphStepId.nullable() }))
    })
  ),
  problem: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const DeploymentGraphListSchema = z.array(DeploymentGraphSchema)
