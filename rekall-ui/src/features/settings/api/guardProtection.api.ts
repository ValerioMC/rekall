import { apiClient, request } from '../../../api/client'
import { GuardProtectionSchema } from './schemas/guardProtection.schema'
import type { GuardProtection } from '@/features/settings/model/guardProtection'

export async function fetchGuardProtection(): Promise<GuardProtection> {
  return request(async () => GuardProtectionSchema.parse(await apiClient('/api/guard/status')))
}

export async function installGuardProtection(): Promise<GuardProtection> {
  return request(async () =>
    GuardProtectionSchema.parse(await apiClient('/api/guard/install', { method: 'POST' }))
  )
}

export async function removeGuardProtection(): Promise<GuardProtection> {
  return request(async () =>
    GuardProtectionSchema.parse(await apiClient('/api/guard/uninstall', { method: 'POST' }))
  )
}
