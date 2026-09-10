import { z } from 'zod'
import { CLAUDE_USAGE_SEVERITIES, CLAUDE_USAGE_STATUSES } from '@/model/claude'

export const ClaudeInstallationSchema = z.object({
  status: z.enum(['CONNECTED', 'OUTDATED', 'NOT_CONNECTED', 'CLI_MISSING']),
  endpoint: z.string(),
  registeredUrl: z.string().nullable(),
  folderScoped: z.array(z.string()),
  commandInstalled: z.boolean(),
  cliPath: z.string().nullable(),
  manualCommand: z.string()
})

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
