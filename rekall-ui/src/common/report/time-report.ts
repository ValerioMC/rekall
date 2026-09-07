import { dateKey } from '@/common/calendar/month-grid'
import { formatDuration } from '@/common/format/duration'
import { isWithin } from './period'
import type { PeriodRange } from './period'
import type { Company, Task, TaskStep, TimeEntry } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

export interface ReportStepRow {
  readonly stepId: TaskStepId
  readonly title: string
  readonly doneAt: Date
}

export interface ReportTaskRow {
  readonly taskId: TaskId
  readonly title: string
  readonly anchor: string
  readonly totalSeconds: number
  readonly isRunning: boolean
  readonly perDaySeconds: readonly number[]
  readonly closedSteps: readonly ReportStepRow[]
  readonly openStepCount: number
  readonly doneElsewhereCount: number
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
  readonly share: number
  readonly projects: readonly ReportProjectRow[]
}

export interface ReportDay {
  readonly date: Date
  readonly totalSeconds: number
  readonly byCompany: readonly { readonly companyId: CompanyId; readonly seconds: number }[]
}

export interface TimeReport {
  readonly totalSeconds: number
  readonly companies: readonly ReportCompanyRow[]
  readonly days: readonly ReportDay[]
  readonly busiestDay: ReportDay | null
  readonly taskCount: number
  readonly closedStepCount: number
}

export function buildTimeReport(
  entries: readonly TimeEntry[],
  tasks: readonly Task[],
  steps: readonly TaskStep[],
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

    const task = taskById.get(entry.taskId)
    const company = task ? companyByName.get(task.companyName) : undefined
    if (!task || !company) continue
    if (selected.size > 0 && !selected.has(company.id)) continue

    const seconds = elapsedSeconds(entry, nowMs)
    if (seconds <= 0) continue

    perDay[index]!.set(company.id, (perDay[index]!.get(company.id) ?? 0) + seconds)
    accumulate(accumulators, company, task, entry, seconds, index, range.days.length)
  }

  const stepsByTask = groupSteps(steps, range)
  const companyRows = [...accumulators.values()]
    .map((accumulator) => toCompanyRow(accumulator, stepsByTask))
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
    taskCount: everyTask(companyRows).length,
    closedStepCount: everyTask(companyRows).reduce(
      (count, task) => count + task.closedSteps.length,
      0
    )
  }
}

function everyTask(companies: readonly ReportCompanyRow[]): readonly ReportTaskRow[] {
  return companies.flatMap((company) => company.projects.flatMap((project) => project.tasks))
}

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

interface StepSummary {
  readonly closed: readonly ReportStepRow[]
  readonly openCount: number
  readonly doneElsewhereCount: number
}

const NO_STEPS: StepSummary = { closed: [], openCount: 0, doneElsewhereCount: 0 }

function groupSteps(
  steps: readonly TaskStep[],
  range: PeriodRange
): Map<TaskId, StepSummary> {
  const byTask = new Map<TaskId, { closed: ReportStepRow[]; openCount: number; doneElsewhereCount: number }>()

  for (const step of steps) {
    let summary = byTask.get(step.taskId)
    if (!summary) {
      summary = { closed: [], openCount: 0, doneElsewhereCount: 0 }
      byTask.set(step.taskId, summary)
    }

    if (!step.done) {
      summary.openCount += 1
      continue
    }

    const doneAt = step.doneAt ? new Date(step.doneAt) : null
    if (!doneAt || !isWithin(range, doneAt)) {
      summary.doneElsewhereCount += 1
      continue
    }
    summary.closed.push({ stepId: step.id, title: step.title, doneAt })
  }

  for (const summary of byTask.values()) {
    summary.closed.sort((a, b) => a.doneAt.getTime() - b.doneAt.getTime())
  }
  return byTask
}

function toTaskRow(accumulator: TaskAccumulator, steps: StepSummary): ReportTaskRow {
  return {
    taskId: accumulator.taskId,
    title: accumulator.title,
    anchor: accumulator.anchor,
    totalSeconds: accumulator.totalSeconds,
    isRunning: accumulator.isRunning,
    perDaySeconds: accumulator.perDaySeconds,
    closedSteps: steps.closed,
    openStepCount: steps.openCount,
    doneElsewhereCount: steps.doneElsewhereCount
  }
}

function toCompanyRow(
  accumulator: CompanyAccumulator,
  stepsByTask: Map<TaskId, StepSummary>
): ReportCompanyRow {
  const projects = [...accumulator.projects.values()]
    .map((project) => {
      const tasks = [...project.tasks.values()]
        .sort(byTotalDescending)
        .map((task) => toTaskRow(task, stepsByTask.get(task.taskId) ?? NO_STEPS))
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

function elapsedSeconds(entry: TimeEntry, nowMs: number): number {
  const end = entry.stoppedAt ? Date.parse(entry.stoppedAt) : nowMs
  return (end - Date.parse(entry.startedAt)) / 1000
}

export function reportAsMarkdown(report: TimeReport, range: PeriodRange): string {
  const lines: string[] = [`# ${range.period === 'week' ? 'Week' : 'Month'} of ${range.label}`, '']
  const closed =
    report.closedStepCount === 0
      ? ''
      : `, closing ${countOf(report.closedStepCount, 'step')}`
  lines.push(
    `**${formatDuration(report.totalSeconds)}** across ${countOf(report.taskCount, 'task')}${closed}.`
  )

  for (const company of report.companies) {
    lines.push('', `## ${company.name} · ${formatDuration(company.totalSeconds)}`)
    for (const project of company.projects) {
      lines.push('', `### ${project.title} · ${formatDuration(project.totalSeconds)}`)
      for (const task of project.tasks) {
        lines.push(`- **${formatDuration(task.totalSeconds)}** ${task.title} · \`${task.anchor}\``)
        for (const step of task.closedSteps) {
          lines.push(`  - [x] ${step.title} · ${stepDay(step.doneAt)}`)
        }
        const tail = stepTail(task)
        if (tail) lines.push(`  - _${tail}_`)
      }
    }
  }

  if (report.companies.length === 0) {
    lines.push('', 'Nothing was tracked in this period.')
  }
  return `${lines.join('\n')}\n`
}

export function stepTail(task: ReportTaskRow): string | null {
  const parts: string[] = []
  if (task.openStepCount > 0) parts.push(`${countOf(task.openStepCount, 'step')} still open`)
  if (task.doneElsewhereCount > 0) {
    parts.push(`${task.doneElsewhereCount} done outside this period`)
  }
  return parts.length === 0 ? null : parts.join(' · ')
}

export function stepDay(date: Date): string {
  return date.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' })
}

function countOf(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}
