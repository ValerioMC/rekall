import { z } from 'zod'

const envSchema = z.object({
  VITE_API_BASE_URL: z.string().default(''),
  VITE_APP_TITLE: z.string().default('Rekall'),
  MODE: z.enum(['development', 'production', 'test'])
})

export const env = envSchema.parse(import.meta.env)
