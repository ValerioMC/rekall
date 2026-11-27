import { apiClient, request } from '../../../api/client'
import { ClaudeInstallationSchema, ClaudeModelsSchema, ClaudeUsageSchema, CommandUpdateStateSchema } from './schemas/claude.schema'
import type { ClaudeInstallation, ClaudeModel, ClaudeUsage, CommandUpdateState } from '@/features/claude/model/claude'

export async function fetchClaudeInstallation(): Promise<ClaudeInstallation> {
  return request(async () => ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude')))
}

export async function installClaudeIntegration(): Promise<ClaudeInstallation> {
  return request(async () =>
    ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude/install', { method: 'POST' }))
  )
}

export async function fetchClaudeModels(): Promise<ClaudeModel[]> {
  return request(async () => ClaudeModelsSchema.parse(await apiClient('/api/claude/models')))
}

export async function fetchCommandUpdateState(): Promise<CommandUpdateState> {
  return request(async () =>
    CommandUpdateStateSchema.parse(await apiClient('/api/settings/claude/commands-update'))
  )
}

/** `refresh` skips the server's cache: the person asked for a reading, not for the last one. */
export async function fetchClaudeUsage(refresh = false): Promise<ClaudeUsage> {
  return request(async () =>
    ClaudeUsageSchema.parse(
      await apiClient('/api/claude/usage', { query: refresh ? { refresh: true } : {} })
    )
  )
}
