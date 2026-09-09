import { z } from 'zod'
import { asClaudeMessageId, asClaudeSessionId, asTaskId, asTaskStepId } from '@/model/branded'
import {
  CLAUDE_MESSAGE_ROLES,
  CLAUDE_SESSION_STATUSES,
  CLAUDE_USAGE_SEVERITIES,
  CLAUDE_USAGE_STATUSES
} from '@/model/claude'

export const ClaudeInstallationSchema = z.object({
  status: z.enum(['CONNECTED', 'OUTDATED', 'NOT_CONNECTED', 'CLI_MISSING']),
  endpoint: z.string(),
  registeredUrl: z.string().nullable(),
  folderScoped: z.array(z.string()),
  commandInstalled: z.boolean(),
  cliPath: z.string().nullable(),
  manualCommand: z.string()
})

export const ClaudeSessionSchema = z.object({
  id: z.string().uuid().transform(asClaudeSessionId),
  taskId: z.string().uuid().transform(asTaskId),
  taskLabel: z.string(),
  taskTitle: z.string(),
  projectLabel: z.string(),
  anchor: z.string(),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  anchors: z.string(),
  workingDir: z.string(),
  cliSessionId: z.string().nullable(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  status: z.enum(CLAUDE_SESSION_STATUSES),
  live: z.boolean(),
  skipPermissions: z.boolean(),
  detail: z.string().nullable(),
  exitCode: z.number().int().nullable(),
  lastActivityAt: z.string(),
  startedAt: z.string(),
  endedAt: z.string().nullable(),
  updatedAt: z.string()
})

export const ClaudeMessageSchema = z.object({
  id: z.string().uuid().transform(asClaudeMessageId),
  sessionId: z.string().uuid().transform(asClaudeSessionId),
  seq: z.number().int(),
  role: z.enum(CLAUDE_MESSAGE_ROLES),
  content: z.string().nullable(),
  toolName: z.string().nullable(),
  meta: z.string().nullable(),
  createdAt: z.string()
})

export const ClaudeSessionListSchema = z.array(ClaudeSessionSchema)
export const ClaudeMessageListSchema = z.array(ClaudeMessageSchema)

export const ClaudeUsageSchema = z.object({
  status: z.enum(CLAUDE_USAGE_STATUSES),
  limits: z.array(
    z.object({
      key: z.string(),
      label: z.string(),
      percent: z.number(),
      severity: z.enum(CLAUDE_USAGE_SEVERITIES),
      resetsAt: z.string().nullable()
    })
  ),
  fetchedAt: z.string()
})
