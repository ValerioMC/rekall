import { dateKey } from '@/common/calendar/month-grid'
import { formatDuration } from '@/common/format/duration'
import type { PeriodRange } from './period'
import type { Company, Task, TimeEntry } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/** One task, and what it came to over the period. */
export interface ReportTaskRow {
  readonly taskId: TaskId
  readonly title: string
  /** What you would type after `/rk` to reload it. */
  readonly anchor: string
  readonly totalSeconds: number
  /** Still running as this was built, so the total is a moving one. */
  readonly isRunning: boolean
  /** Seconds on each day of the period, in the order the days come. */
  readonly perDaySeconds: readonly number[]
}

export interface ReportProjectRow {
  readonly projectId: ProjectId
  readonly label: string
  readonly title: string
  readonly totalSeconds: number
  readonly tasks: readonly ReportTaskRow[]
}

export interface ReportCompanyRow {
  readonly companyId: CompanyId
  readonly name: string
  readonly totalSeconds: number
  /** This company's share of the period, 0 to 1, for the bar next to the total. */
  readonly share: number
  readonly projects: readonly ReportProjectRow[]
}

/** One day of the period, and who it went to. */
export interface ReportDay {
  readonly date: Date
  readonly totalSeconds: number
  /** Company id to seconds, so a column can be stacked in the same hues as the rows below it. */
  readonly byCompany: readonly { readonly companyId: CompanyId; readonly seconds: number }[]
}

export interface TimeReport {
  readonly totalSeconds: number
  readonly companies: readonly ReportCompanyRow[]
  readonly days: readonly ReportDay[]
  /** The longest day in the period, or null when nothing was tracked at all. */
  readonly busiestDay: ReportDay | null
  /** How many distinct tasks were worked on. The count the sentence above the table uses. */
  readonly taskCount: number
}

/**
 * What was worked on over a period, by company, project and task.
 *
 * <p>Built here rather than on the server because everything it needs is already in the window:
 * the sessions, the tasks and the companies are loaded whole at startup, and a report is a
 * regrouping of them rather than a new question to ask.
 *
 * <p>A session belongs to the day it started on, which is the rule the calendar already uses. A
 * session running now counts up to `nowMs`, so the total moves while the work does.
 *
 * <p>`selected` empty means every company. A filter nobody has touched should show everything,
 * not nothing.
 */
export function buildTimeReport(
  entries: readonly TimeEntry[],
  tasks: readonly Task[],
  companies: readonly Company[],
  range: PeriodRange,
  nowMs: number,
  selected: ReadonlySet<CompanyId> = new Set()
): TimeReport {
  const taskById = new Map(tasks.map((task) => [task.id, task]))
  const companyByName = new Map(companies.map((company) => [company.name, company]))
  const dayIndex = new Map(range.days.map((day, index) => [dateKey(day), index]))

  const accumulators = new Map<CompanyId, CompanyAccumulator>()
  const perDay = range.days.map(() => new Map<CompanyId, number>())

  for (const entry of entries) {
    const started = new Date(entry.startedAt)
    const index = dayIndex.get(dateKey(started))
    if (index === undefined) continue

    // A task carries the company; the session only knows which task it was. A session whose task
    // is gone cannot exist, because deleting a task deletes its sessions, so this is the
    // impossible case rather than a filter.
    const task = taskById.get(entry.taskId)
    const company = task ? companyByName.get(task.companyName) : undefined
    if (!task || !company) continue
    if (selected.size > 0 && !selected.has(company.id)) continue

    const seconds = elapsedSeconds(entry, nowMs)
    if (seconds <= 0) continue

    perDay[index]!.set(company.id, (perDay[index]!.get(company.id) ?? 0) + seconds)
    accumulate(accumulators, company, task, entry, seconds, index, range.days.length)
  }

  const companyRows = [...accumulators.values()]
    .map((accumulator) => toCompanyRow(accumulator))
    .sort(byTotalDescending)
  const totalSeconds = companyRows.reduce((sum, row) => sum + row.totalSeconds, 0)
  const days = range.days.map((date, index) => toDay(date, perDay[index]!))

  return {
    totalSeconds,
    companies: companyRows.map((row) => ({
      ...row,
      share: totalSeconds === 0 ? 0 : row.totalSeconds / totalSeconds
    })),
    days,
    busiestDay: busiest(days),
    taskCount: companyRows.reduce(
      (count, company) =>
        count + company.projects.reduce((tasksSoFar, project) => tasksSoFar + project.tasks.length, 0),
      0
    )
  }
}

// ------------------------------------------------------------------ assembly

interface TaskAccumulator {
  readonly taskId: TaskId
  readonly title: string
  readonly anchor: string
  totalSeconds: number
  isRunning: boolean
  readonly perDaySeconds: number[]
}

interface ProjectAccumulator {
  readonly projectId: ProjectId
  readonly label: string
  readonly title: string
  readonly tasks: Map<TaskId, TaskAccumulator>
}

interface CompanyAccumulator {
  readonly companyId: CompanyId
  readonly name: string
  readonly projects: Map<ProjectId, ProjectAccumulator>
}

function accumulate(
  companies: Map<CompanyId, CompanyAccumulator>,
  company: Company,
  task: Task,
  entry: TimeEntry,
  seconds: number,
  dayIndex: number,
  dayCount: number
): void {
  let companyAccumulator = companies.get(company.id)
  if (!companyAccumulator) {
    companyAccumulator = { companyId: company.id, name: company.name, projects: new Map() }
    companies.set(company.id, companyAccumulator)
  }

  let project = companyAccumulator.projects.get(task.projectId)
  if (!project) {
    project = {
      projectId: task.projectId,
      label: task.projectLabel,
      title: task.projectTitle,
      tasks: new Map()
    }
    companyAccumulator.projects.set(task.projectId, project)
  }

  let row = project.tasks.get(task.id)
  if (!row) {
    row = {
      taskId: task.id,
      title: task.title,
      anchor: task.anchor,
      totalSeconds: 0,
      isRunning: false,
      perDaySeconds: Array<number>(dayCount).fill(0)
    }
    project.tasks.set(task.id, row)
  }

  row.totalSeconds += seconds
  row.isRunning = row.isRunning || entry.stoppedAt === null
  row.perDaySeconds[dayIndex] = (row.perDaySeconds[dayIndex] ?? 0) + seconds
}

function toCompanyRow(accumulator: CompanyAccumulator): ReportCompanyRow {
  const projects = [...accumulator.projects.values()]
    .map((project) => {
      const tasks = [...project.tasks.values()].sort(byTotalDescending)
      return {
        projectId: project.projectId,
        label: project.label,
        title: project.title,
        totalSeconds: tasks.reduce((sum, task) => sum + task.totalSeconds, 0),
        tasks
      }
    })
    .sort(byTotalDescending)

  return {
    companyId: accumulator.companyId,
    name: accumulator.name,
    totalSeconds: projects.reduce((sum, project) => sum + project.totalSeconds, 0),
    share: 0,
    projects
  }
}

function toDay(date: Date, byCompany: Map<CompanyId, number>): ReportDay {
  const entries = [...byCompany.entries()]
    .map(([companyId, seconds]) => ({ companyId, seconds }))
    .sort((a, b) => b.seconds - a.seconds)
  return {
    date,
    totalSeconds: entries.reduce((sum, entry) => sum + entry.seconds, 0),
    byCompany: entries
  }
}

function busiest(days: readonly ReportDay[]): ReportDay | null {
  return days.reduce<ReportDay | null>(
    (best, day) => (day.totalSeconds > (best?.totalSeconds ?? 0) ? day : best),
    null
  )
}

function byTotalDescending(a: { totalSeconds: number }, b: { totalSeconds: number }): number {
  return b.totalSeconds - a.totalSeconds
}

/** A session that is still open counts up to now, the way the running clock does. */
function elapsedSeconds(entry: TimeEntry, nowMs: number): number {
  const end = entry.stoppedAt ? Date.parse(entry.stoppedAt) : nowMs
  return (end - Date.parse(entry.startedAt)) / 1000
}

// ------------------------------------------------------------------ handing it over

/**
 * The same report as markdown, for pasting into an invoice, an email or a note.
 *
 * <p>Every task keeps its anchor, so a line in a report someone is reading back is still one
 * `/rk` away from the work it describes. That is the whole reason this application exists, and a
 * report that dropped it would be a dead end on paper.
 */
export function reportAsMarkdown(report: TimeReport, range: PeriodRange): string {
  const lines: string[] = [`# ${range.period === 'week' ? 'Week' : 'Month'} of ${range.label}`, '']
  lines.push(`**${formatDuration(report.totalSeconds)}** across ${countOf(report.taskCount, 'task')}.`)

  for (const company of report.companies) {
    lines.push('', `## ${company.name} · ${formatDuration(company.totalSeconds)}`)
    for (const project of company.projects) {
      lines.push('', `### ${project.title} · ${formatDuration(project.totalSeconds)}`)
      for (const task of project.tasks) {
        lines.push(`- **${formatDuration(task.totalSeconds)}** ${task.title} · \`${task.anchor}\``)
      }
    }
  }

  if (report.companies.length === 0) {
    lines.push('', 'Nothing was tracked in this period.')
  }
  return `${lines.join('\n')}\n`
}

function countOf(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}
