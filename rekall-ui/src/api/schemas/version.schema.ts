import { z } from 'zod'
import { UPDATE_CHECKS } from '@/model/version'

export const VersionStatusSchema = z.object({
  current: z.string(),
  check: z.enum(UPDATE_CHECKS),
  latest: z
    .object({
      version: z.string(),
      releaseUrl: z.string(),
      downloadUrl: z.string().nullable()
    })
    .nullable()
})
