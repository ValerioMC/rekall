import { z } from 'zod'
import type { ChatEvent, ToolInput } from '../../model/chat-event'

const ToolInputSchema: z.ZodType<ToolInput> = z.discriminatedUnion('tool', [
  z.object({ tool: z.literal('bash'), command: z.string(), description: z.string().nullable() }),
  z.object({
    tool: z.literal('edit'),
    filePath: z.string(),
    oldString: z.string(),
    newString: z.string(),
    replaceAll: z.boolean()
  }),
  z.object({ tool: z.literal('write'), filePath: z.string(), content: z.string() }),
  z.object({ tool: z.literal('read'), filePath: z.string() }),
  z.object({
    tool: z.literal('search'),
    name: z.string(),
    pattern: z.string(),
    path: z.string().nullable()
  }),
  z.object({ tool: z.literal('other'), name: z.string(), text: z.string() })
])

const TurnUsageSchema = z.object({
  inputTokens: z.number(),
  cacheCreationInputTokens: z.number(),
  cacheReadInputTokens: z.number(),
  outputTokens: z.number()
})

export const ChatEventSchema: z.ZodType<ChatEvent> = z.discriminatedUnion('type', [
  z.object({ seq: z.number(), type: z.literal('userMessage'), text: z.string() }),
  z.object({
    seq: z.number(),
    type: z.literal('assistantText'),
    messageId: z.string(),
    text: z.string(),
    complete: z.boolean()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('assistantThinking'),
    messageId: z.string(),
    text: z.string(),
    complete: z.boolean()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('toolUse'),
    toolUseId: z.string(),
    name: z.string(),
    input: ToolInputSchema
  }),
  z.object({
    seq: z.number(),
    type: z.literal('toolResult'),
    toolUseId: z.string(),
    isError: z.boolean(),
    excerpt: z.string(),
    truncated: z.boolean()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('permissionRequested'),
    requestId: z.string(),
    toolUseId: z.string(),
    name: z.string(),
    input: ToolInputSchema
  }),
  z.object({
    seq: z.number(),
    type: z.literal('permissionResolved'),
    requestId: z.string(),
    allowed: z.boolean()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('turnFinished'),
    usage: TurnUsageSchema,
    durationMs: z.number(),
    outcome: z.enum(['completed', 'interrupted', 'failed'])
  }),
  z.object({
    seq: z.number(),
    type: z.literal('notice'),
    kind: z.enum(['apiRetry', 'compaction', 'warning']),
    text: z.string()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('ended'),
    exitCode: z.number(),
    detail: z.string()
  })
])
