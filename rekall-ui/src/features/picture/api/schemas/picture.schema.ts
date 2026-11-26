import { z } from 'zod'

export const ImportedPictureSchema = z.object({ url: z.string() })
