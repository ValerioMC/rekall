import { apiClient, request } from '@/api/client'
import { CodexModelsSchema, CodexSetupSchema, CodexUsageSchema } from '@/features/codex/api/codex.schema'
import type { CodexModel, CodexSetup, CodexUsage } from '@/features/codex/model/codex'

export function fetchCodexModels(): Promise<CodexModel[]> {
  return request(async () => CodexModelsSchema.parse(await apiClient('/api/codex/models')))
}

export function fetchCodexSetup(): Promise<CodexSetup> {
  return request(async () => CodexSetupSchema.parse(await apiClient('/api/settings/codex')))
}

export function installCodexIntegration(): Promise<CodexSetup> {
  return request(async () => CodexSetupSchema.parse(await apiClient('/api/settings/codex/install', { method: 'POST' })))
}

export function fetchCodexUsage(refresh = false): Promise<CodexUsage> {
  return request(async () =>
    CodexUsageSchema.parse(await apiClient('/api/codex/usage', { query: refresh ? { refresh: true } : {} }))
  )
}
