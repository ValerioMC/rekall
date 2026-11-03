import { z } from 'zod'

export const GuardProtectionSchema = z.object({
  state: z.enum(['INSTALLED', 'NOT_INSTALLED', 'INVALID_SETTINGS']),
  settingsFile: z.string(),
  problem: z.string().nullable()
})
