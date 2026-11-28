import { APP_LOCALE } from '@/common/format/app-locale'
import { SESSION_PROVIDERS, type SessionProvider } from '@/features/claude/model/session-provider'
import { CLOSURE_CREDITS } from '@/features/provider-report/model/provider-report'
import type {
  ClosureCount, ClosureCredit, ProviderReport, ProviderReportFigures, ProviderReportInput,
  ProviderReportPeriod, ProviderReportTaskRow, ProviderTokens, ReportStep, ReportTask
} from '@/features/provider-report/model/provider-report'
import { shiftUtcPeriod, utcPeriodStart, type CalendarPeriod } from '@/features/time/lib/period'
import { addTokenSpend } from '@/features/tokens/lib/token-totals'
import { NO_TOKENS, type TokenDay, type TokenTally, type TokenTotals } from '@/features/tokens/model/token-usage'
import type { TaskId } from '@/model/branded'

const SHORT_DATE = new Intl.DateTimeFormat(APP_LOCALE, { day: 'numeric', month: 'short', timeZone: 'UTC' })
const LONG_DATE = new Intl.DateTimeFormat(APP_LOCALE, { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' })
const MONTH = new Intl.DateTimeFormat(APP_LOCALE, { month: 'long', year: 'numeric', timeZone: 'UTC' })

type TokenBuckets = Record<SessionProvider, TokenTotals>
type MutableClosureCount = { tasks: number; steps: number }
type ClosureBuckets = Record<ClosureCredit, MutableClosureCount>
type Spend = Parameters<typeof addTokenSpend>[1]

interface TaskActivity {
  readonly tokens: TokenBuckets
  stepsClosed: number
  included: boolean
  latestAt: number
}

interface PeriodActivity {
  readonly tokens: TokenBuckets
  readonly closures: ClosureBuckets
  readonly tasks: Map<TaskId, TaskActivity>
}

interface TaskHistory {
  readonly task: ReportTask
  readonly closedAt: string | null
  readonly lifetimeTokens: ProviderTokens
  readonly providers: readonly SessionProvider[]
}

type PeriodForDate = (date: string | null) => PeriodActivity

export function tokensConsumed(tokens: TokenTotals): number {
  return tokens.input + tokens.cacheWrite + tokens.cacheRead + tokens.generated
}

export function reportCloseDate(date: string): string {
  return LONG_DATE.format(new Date(date))
}

function periodLabel(grouping: CalendarPeriod, start: string): string {
  if (grouping === 'year') return start.slice(0, 4)
  if (grouping === 'month') return MONTH.format(new Date(start))
  const ending = new Date(shiftUtcPeriod(grouping, start, 1))
  ending.setUTCDate(ending.getUTCDate() - 1)
  const first = new Date(start)
  const firstFormat = first.getUTCFullYear() === ending.getUTCFullYear() ? SHORT_DATE : LONG_DATE
  return `${firstFormat.format(first)} – ${LONG_DATE.format(ending)}`
}

function emptyTokens(): TokenBuckets {
  return { CLAUDE: NO_TOKENS, CODEX: NO_TOKENS }
}

function emptyClosures(): ClosureBuckets {
  return { CLAUDE: { tasks: 0, steps: 0 }, CODEX: { tasks: 0, steps: 0 }, BOTH: { tasks: 0, steps: 0 }, NOT_RECORDED: { tasks: 0, steps: 0 } }
}

function totalClosed(closures: ClosureBuckets): ClosureCount {
  const total = { tasks: 0, steps: 0 }
  for (const credit of CLOSURE_CREDITS) {
    total.tasks += closures[credit].tasks
    total.steps += closures[credit].steps
  }
  return total
}

function figures(tokens: ProviderTokens, closures: ClosureBuckets): ProviderReportFigures {
  return { tokens, closures, totalClosed: totalClosed(closures) }
}

function taskIndex<Entry extends { taskId: TaskId }>(entries: readonly Entry[]): ReadonlyMap<TaskId, readonly Entry[]> {
  const byTask = new Map<TaskId, Entry[]>()
  for (const entry of entries) {
    const batch = byTask.get(entry.taskId)
    if (batch) batch.push(entry)
    else byTask.set(entry.taskId, [entry])
  }
  return byTask
}

function tallyProvider(tally: TokenTally): SessionProvider {
  return tally.provider ?? 'CLAUDE'
}

function recordedProviders(tallies: readonly TokenTally[]): readonly SessionProvider[] {
  const seen = new Set(tallies.map(tallyProvider))
  return SESSION_PROVIDERS.filter((provider) => seen.has(provider))
}

function closureCredit(providers: readonly SessionProvider[]): ClosureCredit {
  return providers.length > 1 ? 'BOTH' : providers[0] ?? 'NOT_RECORDED'
}

function taskClosedAt(task: ReportTask, steps: readonly ReportStep[]): string | null {
  if (task.kind === 'EPIC' || task.status !== 'DONE') return null
  let latest: string | null = null
  for (const step of steps) {
    if (step.state === 'DRAFT' || !step.doneAt) continue
    if (!latest || Date.parse(step.doneAt) > Date.parse(latest)) latest = step.doneAt
  }
  return latest ?? task.acceptedAt
}

function addSpend(buckets: TokenBuckets, provider: SessionProvider, spend: Spend): void {
  buckets[provider] = addTokenSpend(buckets[provider], spend)
}

function recordLifetimeTokens(tallies: readonly TokenTally[], overall: TokenBuckets): ProviderTokens {
  const lifetime = emptyTokens()
  for (const tally of tallies) {
    const provider = tallyProvider(tally)
    addSpend(lifetime, provider, tally)
    addSpend(overall, provider, tally)
  }
  return lifetime
}

function activityFor(period: PeriodActivity, taskId: TaskId, date: string | null): TaskActivity {
  const activity = period.tasks.get(taskId) ?? { tokens: emptyTokens(), stepsClosed: 0, included: false, latestAt: 0 }
  if (date) activity.latestAt = Math.max(activity.latestAt, Date.parse(date))
  period.tasks.set(taskId, activity)
  return activity
}

function recordDays(days: readonly TokenDay[], taskId: TaskId, periodFor: PeriodForDate): void {
  for (const day of days) {
    const period = periodFor(day.day)
    const activity = activityFor(period, taskId, day.day)
    addSpend(activity.tokens, day.provider, day)
    addSpend(period.tokens, day.provider, day)
    activity.included = true
  }
}

function recordStepClosures(
  steps: readonly ReportStep[], tallies: readonly TokenTally[], overall: ClosureBuckets,
  periodFor: PeriodForDate
): void {
  for (const step of steps) {
    if (step.state !== 'DONE') continue
    const credit = closureCredit(recordedProviders(tallies.filter((tally) => tally.stepId === step.id)))
    const period = periodFor(step.doneAt)
    overall[credit].steps++
    period.closures[credit].steps++
    activityFor(period, step.taskId, step.doneAt).stepsClosed++
  }
}

function recordTaskClosure(history: TaskHistory, overall: ClosureBuckets, periodFor: PeriodForDate): void {
  if (history.task.kind === 'EPIC' || history.task.status !== 'DONE') return
  const credit = closureCredit(history.providers)
  const period = periodFor(history.closedAt)
  overall[credit].tasks++
  period.closures[credit].tasks++
  activityFor(period, history.task.id, history.closedAt).included = true
}

function newPeriod(): PeriodActivity {
  return { tokens: emptyTokens(), closures: emptyClosures(), tasks: new Map() }
}

function taskRows(period: PeriodActivity, historyByTask: ReadonlyMap<TaskId, TaskHistory>): ProviderReportTaskRow[] {
  return [...period.tasks.entries()]
    .filter(([, activity]) => activity.included)
    .sort(([, left], [, right]) => right.latestAt - left.latestAt)
    .flatMap(([id, activity]) => {
      const history = historyByTask.get(id)
      if (!history) return []
      return [{
        ...history,
        providers: SESSION_PROVIDERS.filter((provider) => history.providers.includes(provider) || tokensConsumed(activity.tokens[provider]) > 0),
        stepsClosed: activity.stepsClosed,
        periodTokens: activity.tokens
      }]
    })
}

function listPeriods(
  grouping: CalendarPeriod, today: string, byStart: ReadonlyMap<string, PeriodActivity>, undated: PeriodActivity,
  historyByTask: ReadonlyMap<TaskId, TaskHistory>
): ProviderReportPeriod[] {
  const boundaries = [...byStart.keys(), utcPeriodStart(grouping, today)].sort()
  const first = boundaries[0]!
  const periods: ProviderReportPeriod[] = []
  for (let start = boundaries.at(-1)!; start >= first; start = shiftUtcPeriod(grouping, start, -1)) {
    const activity = byStart.get(start) ?? newPeriod()
    periods.push({ start, label: periodLabel(grouping, start), ...figures(activity.tokens, activity.closures), tasks: taskRows(activity, historyByTask) })
  }
  const missingDates = figures(undated.tokens, undated.closures)
  if (missingDates.totalClosed.tasks || missingDates.totalClosed.steps) {
    periods.push({ start: 'undated', label: 'Date not recorded', ...missingDates, tasks: taskRows(undated, historyByTask) })
  }
  return periods
}

export function buildProviderReport(input: ProviderReportInput, grouping: CalendarPeriod, today: string): ProviderReport {
  const tokens = emptyTokens()
  const closures = emptyClosures()
  const byStart = new Map<string, PeriodActivity>()
  const undated = newPeriod()
  const historyByTask = new Map<TaskId, TaskHistory>()
  const taskSteps = taskIndex(input.steps)
  const taskTallies = taskIndex(input.tallies)
  const taskDays = taskIndex(input.days)

  function periodFor(date: string | null): PeriodActivity {
    if (date === null) return undated
    const start = utcPeriodStart(grouping, date)
    const period = byStart.get(start) ?? newPeriod()
    byStart.set(start, period)
    return period
  }

  for (const task of input.tasks) {
    const steps = taskSteps.get(task.id) ?? []
    const tallies = taskTallies.get(task.id) ?? []
    const history = {
      task, closedAt: taskClosedAt(task, steps),
      lifetimeTokens: recordLifetimeTokens(tallies, tokens), providers: recordedProviders(tallies)
    }
    historyByTask.set(task.id, history)
    recordDays(taskDays.get(task.id) ?? [], task.id, periodFor)
    recordStepClosures(steps, tallies, closures, periodFor)
    recordTaskClosure(history, closures, periodFor)
  }
  return { overall: figures(tokens, closures), periods: listPeriods(grouping, today, byStart, undated, historyByTask) }
}
