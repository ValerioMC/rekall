import { z } from 'zod'
import { CLAUDE_USAGE_SEVERITIES, CLAUDE_USAGE_STATUSES, COMMAND_UPDATE_STATUSES } from '@/features/claude/model/claude'

export const ClaudeInstallationSchema = z.object({
  status: z.enum(['CONNECTED', 'OUTDATED', 'NOT_CONNECTED', 'CLI_MISSING']),
  endpoint: z.string(),
  registeredUrl: z.string().nullable(),
  folderScoped: z.array(z.string()),
  commandInstalled: z.boolean(),
  cliPath: z.string().nullable(),
  manualCommand: z.string()
})

export const CommandUpdateStateSchema = z.object({
  state: z.enum(COMMAND_UPDATE_STATUSES),
  reason: z.string().optional()
})

export const ClaudeModelsSchema = z.array(
  z.object({
    value: z.string(),
    displayName: z.string(),
    resolvedModel: z.string().nullable(),
    supportsEffort: z.boolean().nullable(),
    supportedEffortLevels: z.array(z.string()).nullable()
  })
)

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
  fetchedAt: z.string(),
  retryAt: z.string().nullable()
})
