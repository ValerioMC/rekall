import { apiClient, request } from './client'
import { VersionStatusSchema } from './schemas/version.schema'
import type { VersionStatus } from '@/model/version'

export async function fetchVersionStatus(): Promise<VersionStatus> {
  return request(async () => VersionStatusSchema.parse(await apiClient('/api/version')))
}
