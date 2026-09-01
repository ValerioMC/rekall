import { apiClient, request } from './client'
import { ClaudeInstallationSchema } from './schemas/claude.schema'
import type { ClaudeInstallation } from '@/model/claude'

export async function fetchClaudeInstallation(): Promise<ClaudeInstallation> {
  return request(async () => ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude')))
}

/** Rewrites the registration from scratch, whatever state it was in, and reports the result. */
export async function installClaudeIntegration(): Promise<ClaudeInstallation> {
  return request(async () =>
    ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude/install', { method: 'POST' }))
  )
}
