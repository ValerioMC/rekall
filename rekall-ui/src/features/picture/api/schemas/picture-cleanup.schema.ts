import { z } from 'zod'

const percentage = z.number().int().min(0).max(100)
const count = z.number().int().nonnegative()

const UnusedPictureSchema = z.object({ url: z.string(), folder: z.string(), sizeBytes: count })

export const PictureCleanupStatusSchema = z.discriminatedUnion('state', [
  z.object({ state: z.literal('IDLE') }),
  z.object({ state: z.literal('SEARCHING'), percentage }),
  z.object({ state: z.literal('READY'), pictures: z.array(UnusedPictureSchema), totalBytes: count }),
  z.object({ state: z.literal('DELETING'), percentage }),
  z.object({ state: z.literal('FINISHED'), removed: count, freedBytes: count, kept: count, notDeleted: count }),
  z.object({ state: z.literal('FAILED'), message: z.string() })
])

export const PictureCleanupStreamEventSchema = z.object({
  state: z.enum(['IDLE', 'SEARCHING', 'READY', 'DELETING', 'FINISHED', 'FAILED']),
  percentage,
  count
})
