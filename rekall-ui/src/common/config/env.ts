import { z } from 'zod'

/**
 * Configuration is validated at boot. A missing or malformed variable stops the application
 * immediately rather than surfacing as an unexplained failure on the first request.
 */
const envSchema = z.object({
  // Same origin by default: the Spring Boot jar serves both the UI and the API.
  VITE_API_BASE_URL: z.string().default(''),
  VITE_APP_TITLE: z.string().default('Rekall'),
  MODE: z.enum(['development', 'production', 'test'])
})

export const env = envSchema.parse(import.meta.env)
