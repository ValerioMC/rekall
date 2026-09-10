import { apiClient, request } from './client'
import { ClaudeInstallationSchema, ClaudeUsageSchema } from './schemas/claude.schema'
import type { ClaudeInstallation, ClaudeUsage } from '@/model/claude'

export async function fetchClaudeInstallation(): Promise<ClaudeInstallation> {
  return request(async () => ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude')))
}

export async function installClaudeIntegration(): Promise<ClaudeInstallation> {
  return request(async () =>
    ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude/install', { method: 'POST' }))
  )
}

export async function fetchClaudeUsage(): Promise<ClaudeUsage> {
  return request(async () => ClaudeUsageSchema.parse(await apiClient('/api/claude/usage')))
}
