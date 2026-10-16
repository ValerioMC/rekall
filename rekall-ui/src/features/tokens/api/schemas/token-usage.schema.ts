import { z } from 'zod'
import { asTaskId, asTaskStepId } from '@/model/branded'

export const TokenTallySchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  inputTokens: z.number().int().nonnegative(),
  outputTokens: z.number().int().nonnegative(),
  cacheCreationTokens: z.number().int().nonnegative(),
  cacheReadTokens: z.number().int().nonnegative()
})

export const TokenTallyListSchema = z.array(TokenTallySchema)

export const TokenDaySchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  day: z.string().datetime().transform((instant) => instant.slice(0, 10)),
  inputTokens: z.number().int().nonnegative(),
  outputTokens: z.number().int().nonnegative(),
  cacheCreationTokens: z.number().int().nonnegative(),
  cacheReadTokens: z.number().int().nonnegative()
})

export const TokenDayListSchema = z.array(TokenDaySchema)

export const TokenUsageStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  tallies: TokenTallyListSchema,
  days: TokenDayListSchema
})
