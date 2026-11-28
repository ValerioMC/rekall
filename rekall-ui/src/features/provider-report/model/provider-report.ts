import type { Task } from '@/features/catalog/model/catalog'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TokenDay, TokenTally, TokenTotals } from '@/features/tokens/model/token-usage'

export const CLOSURE_CREDITS = ['CLAUDE', 'CODEX', 'BOTH', 'NOT_RECORDED'] as const
export type ClosureCredit = (typeof CLOSURE_CREDITS)[number]

export const CLOSURE_CREDIT_LABEL: Readonly<Record<'BOTH' | 'NOT_RECORDED', string>> = {
  BOTH: 'Both',
  NOT_RECORDED: 'Not recorded'
}

export type ReportTask = Pick<Task, 'id' | 'code' | 'title' | 'projectTitle' | 'companyName' | 'status' | 'kind' | 'acceptedAt'>
export type ReportStep = Pick<TaskStep, 'id' | 'taskId' | 'state' | 'doneAt'>
export type ProviderTokens = Readonly<Record<SessionProvider, TokenTotals>>
export type ClosureCount = Readonly<{ tasks: number; steps: number }>

export interface ProviderReportInput {
  readonly tasks: readonly ReportTask[]
  readonly steps: readonly ReportStep[]
  readonly tallies: readonly TokenTally[]
  readonly days: readonly TokenDay[]
}

export interface ProviderReportFigures {
  readonly tokens: ProviderTokens
  readonly closures: Readonly<Record<ClosureCredit, ClosureCount>>
  readonly totalClosed: ClosureCount
}

export interface ProviderReportTaskRow {
  readonly task: ReportTask
  readonly closedAt: string | null
  readonly stepsClosed: number
  readonly providers: readonly SessionProvider[]
  readonly periodTokens: ProviderTokens
  readonly lifetimeTokens: ProviderTokens
}

export interface ProviderReportPeriod extends ProviderReportFigures {
  readonly start: string
  readonly label: string
  readonly tasks: readonly ProviderReportTaskRow[]
}

export interface ProviderReport {
  readonly overall: ProviderReportFigures
  readonly periods: readonly ProviderReportPeriod[]
}
