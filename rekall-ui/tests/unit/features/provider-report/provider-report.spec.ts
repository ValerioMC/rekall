import { describe, expect, it } from 'vitest'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import { buildProviderReport, tokensConsumed } from '@/features/provider-report/lib/provider-report'
import type { ProviderReportInput, ReportStep, ReportTask } from '@/features/provider-report/model/provider-report'
import type { CalendarPeriod } from '@/features/time/lib/period'
import type { TokenDay, TokenTally } from '@/features/tokens/model/token-usage'
import { asTaskId, asTaskStepId } from '@/model/branded'

const spend = { inputTokens: 11, cacheCreationTokens: 23, cacheReadTokens: 37, outputTokens: 51 }

function task(code: string, options: Partial<ReportTask> = {}): ReportTask {
  return { id: asTaskId(code), code, title: code, companyName: 'Acme', projectTitle: 'Vega', kind: 'TASK', status: 'TODO', acceptedAt: null, ...options }
}

function step(owner: ReportTask, id: string, doneAt: string, state: ReportStep['state'] = 'DONE'): ReportStep {
  return { id: asTaskStepId(id), taskId: owner.id, state, doneAt }
}

function tally(task: ReportTask, provider: SessionProvider, step?: ReportStep): TokenTally {
  return { taskId: task.id, stepId: step?.id ?? null, provider, ...spend }
}

function day(task: ReportTask, provider: SessionProvider, date: string): TokenDay {
  return { taskId: task.id, provider, day: date, ...spend }
}

function fixture(): ProviderReportInput {
  const both = task('BOTH', { status: 'DONE', acceptedAt: '2026-10-01T12:00:00Z' })
  const claude = task('CLAUDE', { status: 'DONE', acceptedAt: '2026-11-30T23:30:00Z' })
  const codex = task('CODEX', { status: 'DONE', acceptedAt: '2026-12-01T00:30:00Z' })
  const unrecorded = task('UNRECORDED', { status: 'DONE', acceptedAt: '2026-11-28T12:00:00Z' })
  const epic = task('EPIC', { kind: 'EPIC', status: 'DONE', acceptedAt: '2026-11-28T12:00:00Z' })
  const open = task('OPEN')
  const deleted = task('DELETED')
  const claudeStep = step(both, 's1', '2026-11-23T00:00:00Z')
  const codexStep = step(both, 's2', '2026-11-29T23:59:59Z')
  const sharedStep = step(both, 's3', '2026-11-30T23:59:59Z')
  const unrecordedStep = step(unrecorded, 's4', '2026-11-28T12:00:00Z')
  return {
    tasks: [both, claude, codex, unrecorded, epic, open],
    steps: [claudeStep, codexStep, sharedStep, unrecordedStep, step(both, 'draft', '2026-12-02T01:00:00Z', 'DRAFT'), step(open, 'claim', '2026-11-30T12:00:00Z', 'CLAIMED')],
    tallies: [tally(both, 'CLAUDE', claudeStep), tally(both, 'CODEX', codexStep), tally(both, 'CLAUDE', sharedStep), tally(both, 'CODEX', sharedStep), tally(claude, 'CLAUDE'), tally(codex, 'CODEX'), tally(deleted, 'CODEX')],
    days: [day(both, 'CLAUDE', '2026-11-23'), day(both, 'CODEX', '2026-11-29'), day(both, 'CLAUDE', '2026-11-30'), day(both, 'CODEX', '2026-12-01'), day(claude, 'CLAUDE', '2026-11-30'), day(codex, 'CODEX', '2026-12-01'), day(deleted, 'CODEX', '2026-12-01')]
  }
}

describe('provider report figures', () => {
  it('credits each closure once using its own tallies, skips drafts and claimed steps, and excludes epics', () => {
    const report = buildProviderReport(fixture(), 'month', '2026-12-01')

    expect(report.overall.closures).toEqual({
      CLAUDE: { tasks: 1, steps: 1 }, CODEX: { tasks: 1, steps: 1 },
      BOTH: { tasks: 1, steps: 1 }, NOT_RECORDED: { tasks: 1, steps: 1 }
    })
    expect(report.overall.totalClosed).toEqual({ tasks: 4, steps: 4 })
    expect(report.periods[1]).toMatchObject({ start: '2026-11-01', totalClosed: { tasks: 3, steps: 4 } })
    expect(report.periods[0]?.tasks.map(({ task }) => task.code)).toEqual(['CODEX', 'BOTH'])
  })

  it('shows each provider’s tokens in the period and over the lifetime of a task', () => {
    const report = buildProviderReport(fixture(), 'month', '2026-12-01')
    const november = report.periods[1]
    const both = november?.tasks.find(({ task }) => task.code === 'BOTH')

    expect(tokensConsumed(report.overall.tokens.CLAUDE)).toBe(366)
    expect(november?.tokens.CLAUDE).toEqual({ input: 33, cacheWrite: 69, cacheRead: 111, generated: 153 })
    expect(both).toMatchObject({ providers: ['CLAUDE', 'CODEX'], stepsClosed: 3, closedAt: '2026-11-30T23:59:59Z' })
    expect(both && tokensConsumed(both.periodTokens.CODEX)).toBe(122)
    expect(both && tokensConsumed(both.lifetimeTokens.CODEX)).toBe(244)
    expect(november?.tasks.some(({ task }) => task.code === 'UNRECORDED')).toBe(true)
    expect(november?.tasks.some(({ task }) => task.code === 'EPIC')).toBe(false)
  })

  it.each(['week', 'month', 'year'] as const)('reconciles %s figures against overall totals', (grouping) => {
    const { overall, periods } = buildProviderReport(fixture(), grouping, '2026-12-01')
    for (const provider of ['CLAUDE', 'CODEX'] as const) {
      for (const kind of ['input', 'cacheWrite', 'cacheRead', 'generated'] as const) {
        expect(periods.reduce((total, period) => total + period.tokens[provider][kind], 0)).toBe(overall.tokens[provider][kind])
      }
    }
    for (const credit of ['CLAUDE', 'CODEX', 'BOTH', 'NOT_RECORDED'] as const) {
      for (const kind of ['tasks', 'steps'] as const) {
        expect(periods.reduce((total, period) => total + period.closures[credit][kind], 0)).toBe(overall.closures[credit][kind])
      }
    }
  })

  it.each<{ grouping: CalendarPeriod; start: string; end: string; next: string }>([
    { grouping: 'week', start: '2026-11-23', end: '2026-11-29', next: '2026-11-30' },
    { grouping: 'month', start: '2026-11-01', end: '2026-11-30', next: '2026-12-01' },
    { grouping: 'year', start: '2026-01-01', end: '2026-12-31', next: '2027-01-01' }
  ])('places the first and last UTC days of a $grouping together', ({ grouping, start, end, next }) => {
    const ended = task('ENDED', { status: 'DONE', acceptedAt: `${next}T00:59:59+01:00` })
    const input = { tasks: [ended], steps: [], tallies: [], days: [day(ended, 'CLAUDE', start), day(ended, 'CLAUDE', end), day(ended, 'CLAUDE', next)] }
    const { periods } = buildProviderReport(input, grouping, next)

    expect(periods.map((period) => [period.start, tokensConsumed(period.tokens.CLAUDE), period.totalClosed.tasks])).toEqual([
      [next, 122, 0], [start, 244, 1]
    ])
  })

  it('lists empty periods newest first and counts a step accepted after its task last consumed tokens', () => {
    const owner = task('OPEN')
    const accepted = step(owner, 'accepted', '2026-11-01T00:01:00Z')
    const input = { tasks: [owner], steps: [accepted], tallies: [tally(owner, 'CODEX', accepted)], days: [day(owner, 'CODEX', '2026-09-01')] }
    const { periods } = buildProviderReport(input, 'month', '2026-11-01')

    expect(periods.map((period) => [period.start, period.totalClosed.steps, period.tasks.length])).toEqual([
      ['2026-11-01', 1, 0], ['2026-10-01', 0, 0], ['2026-09-01', 0, 1]
    ])
  })
})
