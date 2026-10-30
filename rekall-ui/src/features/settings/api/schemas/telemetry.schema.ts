import { z } from 'zod'

export const TelemetrySettingsSchema = z.object({
  enabled: z.boolean(),
  host: z.string(),
  publicKey: z.string(),
  logContent: z.boolean(),
  secretSaved: z.boolean()
})
