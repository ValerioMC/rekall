import { apiClient, request } from '@/api/client'
import { env } from '@/common/config/env'
import { PictureCleanupStatusSchema } from './schemas/picture-cleanup.schema'
import type { PictureCleanupStatus } from '@/features/picture/model/picture-cleanup'

export async function fetchPictureCleanup(): Promise<PictureCleanupStatus> {
  return request(async () => PictureCleanupStatusSchema.parse(await apiClient('/api/image-cleanup')))
}

export async function startPictureSearch(): Promise<void> {
  await request(() => apiClient('/api/image-cleanup', { method: 'POST', retry: 0 }))
}

/** Deletes the pictures the search listed, in the background. */
export async function startPictureDeletion(): Promise<void> {
  await request(() => apiClient('/api/image-cleanup/deletion', { method: 'POST', retry: 0 }))
}

export async function discardPictureList(): Promise<void> {
  await request(() => apiClient('/api/image-cleanup', { method: 'DELETE', retry: 0 }))
}

/** Where a kept picture is served from, by the address the markdown shows it with. */
export function keptPictureUrl(address: string): string {
  return `${env.VITE_API_BASE_URL}${address}`
}
