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
  z.object({
    tool: z.literal('todos'),
    items: z.array(
      z.object({
        content: z.string(),
        status: z.enum(['pending', 'inProgress', 'completed'])
      })
    )
  }),
  z.object({
    tool: z.literal('agent'),
    description: z.string(),
    subagentType: z.string().nullable(),
    prompt: z.string()
  }),
  z.object({ tool: z.literal('web'), name: z.string(), target: z.string() }),
  z.object({
    tool: z.literal('mcp'),
    server: z.string(),
    toolName: z.string(),
    text: z.string()
  }),
  z.object({ tool: z.literal('other'), name: z.string(), text: z.string() })
])

const TurnUsageSchema = z.object({
  inputTokens: z.number(),
  cacheCreationInputTokens: z.number(),
  cacheReadInputTokens: z.number(),
  outputTokens: z.number()
})

const ParentToolUseIdSchema = z.string().nullable()

const TurnContextSchema = z.object({ usedTokens: z.number(), windowTokens: z.number() })

export const ChatEventSchema: z.ZodType<ChatEvent> = z.discriminatedUnion('type', [
  z.object({
    seq: z.number(),
    type: z.literal('sessionStarted'),
    model: z.string(),
    cwd: z.string(),
    permissionMode: z.string(),
    version: z.string()
  }),
  z.object({
    seq: z.number(),
    type: z.literal('commandOutput'),
    command: z.string(),
    text: z.string()
  }),
  z.object({ seq: z.number(), type: z.literal('userMessage'), text: z.string() }),
  z.object({
    seq: z.number(),
    type: z.literal('assistantText'),
    messageId: z.string(),
    text: z.string(),
    complete: z.boolean(),
    parentToolUseId: ParentToolUseIdSchema
  }),
  z.object({
    seq: z.number(),
    type: z.literal('assistantThinking'),
    messageId: z.string(),
    text: z.string(),
    complete: z.boolean(),
    parentToolUseId: ParentToolUseIdSchema
  }),
  z.object({
    seq: z.number(),
    type: z.literal('toolUse'),
    toolUseId: z.string(),
    name: z.string(),
    input: ToolInputSchema,
    parentToolUseId: ParentToolUseIdSchema
  }),
  z.object({
    seq: z.number(),
    type: z.literal('toolResult'),
    toolUseId: z.string(),
    isError: z.boolean(),
    excerpt: z.string(),
    truncated: z.boolean(),
    summary: z.string().nullable(),
    parentToolUseId: ParentToolUseIdSchema
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
    outcome: z.enum(['completed', 'interrupted', 'failed']),
    costUsd: z.number().nullable(),
    context: TurnContextSchema.nullable()
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
