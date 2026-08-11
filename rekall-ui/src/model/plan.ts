import type { PlanId } from './branded'

export type ChangeClass = 'SAFE' | 'NEEDS_INPUT' | 'BLOCKED'

export type PlanPhase =
  | 'CREATE_TABLE'
  | 'TABLE_INFRASTRUCTURE'
  | 'ALTER_COLUMN'
  | 'ADD_CONSTRAINT'
  | 'JOIN_TABLE'
  | 'DROP'

export type PlanStatement = Readonly<{
  phase: PlanPhase
  changeClass: ChangeClass
  sql: string | null
  description: string
  warning: string | null
  /** Identifies the answer this step waits on: `table` or `table.column`. */
  inputKey: string | null
}>

export type Plan = Readonly<{
  planId: PlanId
  applicable: boolean
  blockedCount: number
  awaitingInputCount: number
  statements: readonly PlanStatement[]
}>

export type PlanAnswers = Readonly<{
  backfillDefaults: Readonly<Record<string, string>>
  confirmedDrops: readonly string[]
}>

export type ApplyResult = Readonly<{
  planId: PlanId
  statementCount: number
  documentsDeleted: number
}>

/**
 * A step that waits on an answer is either a value to write into existing rows or a
 * confirmation to destroy something. The two need different controls, and the distinction is
 * not in the payload, so it is derived from the change itself.
 */
export function isDestructive(statement: PlanStatement): boolean {
  return statement.inputKey !== null && statement.description.toLowerCase().startsWith('drop')
}
