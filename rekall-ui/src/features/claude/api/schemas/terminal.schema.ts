import { z } from 'zod'
import { SESSION_PROVIDERS } from '@/features/claude/model/session-provider'
import { ConnectionKindSchema } from '@/features/connection/api/schemas/connection.schema'
import { asConnectionId, asProjectId, asTaskId, asTaskStepId, asTerminalId } from '@/model/branded'

export const TerminalSchema = z.object({
  id: z.string().uuid().transform(asTerminalId),
  projectId: z.string().uuid().transform(asProjectId),
  taskId: z.string().uuid().transform(asTaskId).nullable(),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  anchors: z.string(),
  workingDir: z.string(),
  projectLabel: z.string(),
  taskCode: z.string().nullable(),
  taskTitle: z.string().nullable(),
  mode: z.enum(['WORK', 'PLAN', 'PLAN_FLOW', 'REPLAN', 'REVISE', 'REFINE', 'REWRITE_FIX', 'REWRITE_RESTRUCTURE', 'GENERATE', 'EXPLAIN', 'INVENTORY', 'ASSESS', 'ASSESS_ALL', 'AUTOCONFIGURE', 'REORGANIZE', 'SCAN_COMPONENTS', 'MERGE']),
  provider: z.enum(SESSION_PROVIDERS).default('CLAUDE'),
  skipPermissions: z.boolean(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  kind: z.enum(['TERMINAL', 'CHAT']).default('TERMINAL'),
  live: z.boolean(),
  awaitingPermission: z.boolean().default(false),
  startedAt: z.string(),
  lastActivityAt: z.string(),
  connections: z
    .array(
      z.object({
        connectionId: z.string().uuid().transform(asConnectionId),
        label: z.string(),
        kind: ConnectionKindSchema,
        credentialSet: z.boolean()
      })
    )
    .default([])
})

export const TerminalListSchema = z.array(TerminalSchema)
