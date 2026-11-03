import { z } from 'zod'
import { asTaskId, asWrapupVersionId } from '@/model/branded'

export const WrapupVersionSchema = z.object({
  id: z.string().uuid().transform(asWrapupVersionId),
  taskId: z.string().uuid().transform(asTaskId),
  number: z.number().int(),
  writtenBy: z.enum(['CLAUDE', 'HAND']),
  writtenAt: z.string(),
  length: z.number().int(),
  preview: z.string(),
  commitHash: z.string().nullable()
})

export const WrapupVersionListSchema = z.array(WrapupVersionSchema)

export const WrapupVersionDetailSchema = WrapupVersionSchema.extend({ bodyMarkdown: z.string() })

export const WrapupFreshnessSchema = z.discriminatedUnion('state', [
  z.object({ state: z.literal('fresh') }),
  z.object({ state: z.literal('unknown') }),
  z.object({ state: z.literal('possiblyStale'), changedPaths: z.array(z.string()), commits: z.array(z.string()) })
])
