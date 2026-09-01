import { z } from 'zod'

export const ClaudeInstallationSchema = z.object({
  status: z.enum(['CONNECTED', 'OUTDATED', 'NOT_CONNECTED', 'CLI_MISSING']),
  endpoint: z.string(),
  registeredUrl: z.string().nullable(),
  folderScoped: z.array(z.string()),
  commandInstalled: z.boolean(),
  cliPath: z.string().nullable(),
  manualCommand: z.string()
})
