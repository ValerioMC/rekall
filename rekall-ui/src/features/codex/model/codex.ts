import type { z } from 'zod'
import type { CodexModelsSchema, CodexSetupSchema, CodexUsageSchema, CodexUsageWindowSchema } from '@/features/codex/api/codex.schema'

export type CodexModel = z.infer<typeof CodexModelsSchema>[number]
export type CodexSetup = z.infer<typeof CodexSetupSchema>
export type CodexUsage = z.infer<typeof CodexUsageSchema>
export type CodexUsageWindow = z.infer<typeof CodexUsageWindowSchema>
