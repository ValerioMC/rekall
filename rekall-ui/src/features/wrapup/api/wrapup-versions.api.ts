import { apiClient, request } from '../../../api/client'
import { WrapupVersionDetailSchema, WrapupVersionListSchema } from './schemas/wrapup-version.schema'
import type { WrapupVersion, WrapupVersionDetail } from '@/features/wrapup/model/wrapup-version'
import type { TaskId, WrapupVersionId } from '@/model/branded'

export async function fetchWrapupVersions(taskId: TaskId): Promise<WrapupVersion[]> {
  return request(async () =>
    WrapupVersionListSchema.parse(await apiClient(`/api/tasks/${taskId}/wrapup/versions`))
  )
}

export async function fetchWrapupVersion(taskId: TaskId, versionId: WrapupVersionId): Promise<WrapupVersionDetail> {
  return request(async () =>
    WrapupVersionDetailSchema.parse(await apiClient(`/api/tasks/${taskId}/wrapup/versions/${versionId}`))
  )
}
