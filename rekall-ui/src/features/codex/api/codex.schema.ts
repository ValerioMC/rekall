import { z } from 'zod'

export const CodexModelsSchema = z.array(z.object({
  model: z.string(),
  displayName: z.string(),
  supportedReasoningEfforts: z.array(z.object({ reasoningEffort: z.string() }))
}))

export const CodexSetupSchema = z.object({
  status: z.enum(['CONNECTED', 'NOT_CONNECTED', 'CLI_MISSING'])
})

export const CodexUsageWindowSchema = z.object({
  usedPercent: z.number(),
  resetsAt: z.number().nullable(),
  windowDurationMins: z.number().nullable()
})

export const CodexUsageSchema = z.discriminatedUnion('status', [
  z.object({ status: z.literal('CLI_MISSING') }),
  z.object({ status: z.literal('UNAVAILABLE') }),
  z.object({
    status: z.literal('OK'),
    primary: CodexUsageWindowSchema.nullable(),
    secondary: CodexUsageWindowSchema.nullable()
  })
])
