import { apiClient, request } from './client'
import { VersionStatusSchema } from './schemas/version.schema'
import type { VersionStatus } from '@/model/version'

/** `refresh` asks GitHub again instead of taking the answer the server remembered. */
export async function fetchVersionStatus(refresh = false): Promise<VersionStatus> {
  return request(async () =>
    VersionStatusSchema.parse(await apiClient('/api/version', { query: refresh ? { refresh: true } : undefined }))
  )
}
