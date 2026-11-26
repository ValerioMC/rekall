import { apiClient, request } from '@/api/client'
import { ImportedPictureSchema } from './schemas/picture.schema'
import type { ImportedPicture, PictureOwner } from '@/features/picture/model/picture'

/** Copies a picture of this machine into its owner's folder and answers the address the markdown holds. */
export async function importPicture(sourcePath: string, owner: PictureOwner): Promise<ImportedPicture> {
  return request(async () =>
    ImportedPictureSchema.parse(await apiClient('/api/images', { method: 'POST', body: { sourcePath, owner }, retry: 0 }))
  )
}
