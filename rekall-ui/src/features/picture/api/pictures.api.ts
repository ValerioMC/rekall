import { apiClient, request } from '@/api/client'
import { env } from '@/common/config/env'
import { ImportedPictureSchema } from './schemas/picture.schema'
import type { ImportedPicture, PictureOwner } from '@/features/picture/model/picture'

/** Copies a picture of this machine into its owner's folder and answers the address the markdown holds. */
export async function importPicture(sourcePath: string, owner: PictureOwner): Promise<ImportedPicture> {
  return request(async () =>
    ImportedPictureSchema.parse(await apiClient('/api/images', { method: 'POST', body: { sourcePath, owner }, retry: 0 }))
  )
}

/** Where the preview of a picture file of this machine is served from, before it is copied anywhere. */
export function sourceThumbnailUrl(path: string): string {
  return `${env.VITE_API_BASE_URL}/api/image-thumbnail?path=${encodeURIComponent(path)}`
}

