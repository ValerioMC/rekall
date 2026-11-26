import { apiClient, request } from '../../../api/client'
import { GuardProtectionSchema } from './schemas/guardProtection.schema'
import type { GuardProtection } from '@/features/settings/model/guardProtection'
import type { SessionProvider } from '@/features/claude/model/session-provider'

export async function fetchGuardProtection(provider: SessionProvider): Promise<GuardProtection> {
  return request(async () =>
    GuardProtectionSchema.parse(await apiClient('/api/guard/status', { query: { provider } }))
  )
}

export async function installGuardProtection(provider: SessionProvider): Promise<GuardProtection> {
  return request(async () =>
    GuardProtectionSchema.parse(
      await apiClient('/api/guard/install', { method: 'POST', query: { provider } })
    )
  )
}

export async function removeGuardProtection(provider: SessionProvider): Promise<GuardProtection> {
  return request(async () =>
    GuardProtectionSchema.parse(
      await apiClient('/api/guard/uninstall', { method: 'POST', query: { provider } })
    )
  )
}
