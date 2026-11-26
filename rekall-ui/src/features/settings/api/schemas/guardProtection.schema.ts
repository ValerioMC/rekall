import { z } from 'zod'

export const GuardProtectionSchema = z.object({
  state: z.enum([
    'INSTALLED',
    'NOT_INSTALLED',
    'NEEDS_REPAIR',
    'INVALID_SETTINGS',
    'UNSUPPORTED',
    'TRUST_REQUIRED',
    'DISABLED'
  ]),
  settingsFile: z.string(),
  configured: z.boolean(),
  problem: z.string().nullable()
})
