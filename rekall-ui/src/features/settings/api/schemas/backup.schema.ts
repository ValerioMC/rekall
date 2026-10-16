import { z } from 'zod'

export const BackupFileSchema = z.object({
  name: z.string(),
  sizeBytes: z.number().int(),
  createdAt: z.string(),
  reason: z.enum(['AUTO', 'MANUAL', 'BEFORE_RESTORE', 'UPLOADED'])
})

export const BackupStatusSchema = z.object({
  available: z.boolean(),
  folder: z.string().nullable(),
  intervalHours: z.number().int(),
  keep: z.number().int(),
  backups: z.array(BackupFileSchema),
  lastFailure: z.string().nullable()
})

export const RestoreStartedSchema = z.object({
  restarting: z.boolean(),
  previousState: BackupFileSchema
})
