import { z } from 'zod'
import type { PlanId } from '@/model/branded'

const planId = z.string().uuid().transform((value) => value as PlanId)

export const PlanStatementSchema = z.object({
  phase: z.enum([
    'CREATE_TABLE',
    'TABLE_INFRASTRUCTURE',
    'ALTER_COLUMN',
    'ADD_CONSTRAINT',
    'JOIN_TABLE',
    'DROP'
  ]),
  changeClass: z.enum(['SAFE', 'NEEDS_INPUT', 'BLOCKED']),
  sql: z.string().nullable(),
  description: z.string(),
  warning: z.string().nullable(),
  inputKey: z.string().nullable()
})

export const PlanSchema = z.object({
  planId,
  applicable: z.boolean(),
  blockedCount: z.number().int(),
  awaitingInputCount: z.number().int(),
  statements: z.array(PlanStatementSchema)
})

export const ApplyResultSchema = z.object({
  planId,
  statementCount: z.number().int(),
  documentsDeleted: z.number().int()
})
