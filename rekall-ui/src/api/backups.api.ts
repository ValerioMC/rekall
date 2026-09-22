import { ApiError, apiClient, request } from './client'
import { BackupFileSchema, BackupStatusSchema, RestoreStartedSchema } from './schemas/backup.schema'
import { env } from '@/common/config/env'
import type { BackupFile, BackupStatus } from '@/model/backup'

export async function fetchBackupStatus(): Promise<BackupStatus> {
  return request(async () => BackupStatusSchema.parse(await apiClient('/api/backups')))
}

export async function takeBackup(): Promise<BackupFile> {
  return request(async () => BackupFileSchema.parse(await apiClient('/api/backups', { method: 'POST' })))
}

export function backupDownloadUrl(name: string): string {
  return `${env.VITE_API_BASE_URL}/api/backups/${encodeURIComponent(name)}`
}

/** Restores a listed backup. The server restarts on it; the page has to wait and reload. */
export async function restoreBackup(name: string): Promise<BackupFile> {
  return request(async () =>
    RestoreStartedSchema.parse(
      await apiClient(`/api/backups/${encodeURIComponent(name)}/restore`, { method: 'POST' })
    ).previousState
  )
}

/**
 * Restores a backup file chosen on this machine. Sent with plain fetch: the shared client pins a
 * JSON content type, and a multipart body needs the browser to write its own boundary.
 */
export async function restoreUploadedBackup(file: File): Promise<BackupFile> {
  const form = new FormData()
  form.append('file', file)
  const response = await fetch(`${env.VITE_API_BASE_URL}/api/backups/restore`, { method: 'POST', body: form })
  const body: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const detail = (body as { detail?: string } | null)?.detail ?? 'The restore was refused.'
    throw new ApiError(response.status, detail, response.url)
  }
  return RestoreStartedSchema.parse(body).previousState
}
