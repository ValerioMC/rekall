import { describe, expect, it } from 'vitest'
import { periodRange } from '@/common/report/period'
import { buildTimeReport, reportAsMarkdown } from '@/common/report/time-report'
import type { Company, Task, TimeEntry } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId, TimeEntryId } from '@/model/branded'

/**
 * The regrouping the report page is: sessions in, one client's week out. Every rule worth
 * getting wrong is here — which day a session counts on, what an open one is worth, and what a
 * filter of nothing means.
 */
const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId
const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId
const builder = 't1' as TaskId
const retry = 't2' as TaskId
const signal = 't3' as TaskId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 1, taskCount: 2, updatedAt: '' },
  { id: globex, name: 'globex', description: null, projectCount: 1, taskCount: 1, updatedAt: '' }
]

function task(id: TaskId, title: string, projectId: ProjectId, companyName: string): Task {
  return {
    id,
    label: title.toLowerCase().replace(/ /g, '-'),
    title,
    status: 'IN_PROGRESS',
    description: null,
    projectId,
    projectLabel: projectId === vega ? 'vega' : 'beacon',
    projectTitle: projectId === vega ? 'Vega Platform' : 'Beacon',
    companyName,
    projectRepoFolder: null,
    documentCount: 0,
    hasWrapup: false,
    anchor: `project:${projectId === vega ? 'vega' : 'beacon'} task:${title.toLowerCase().replace(/ /g, '-')}`,
    updatedAt: ''
  }
}

const tasks: Task[] = [
  task(builder, 'Report builder', vega, 'acme'),
  task(retry, 'Retry policy', vega, 'acme'),
  task(signal, 'Signal ingest', beacon, 'globex')
]

let nextId = 0

/** A session on one local day, given in hours so the arithmetic is readable. */
function session(taskId: TaskId, day: Date, hours: number, open = false): TimeEntry {
  const started = new Date(day.getFullYear(), day.getMonth(), day.getDate(), 9, 0)
  const stopped = new Date(started.getTime() + hours * 3600 * 1000)
  const known = tasks.find((candidate) => candidate.id === taskId)!
  return {
    id: `te${nextId++}` as TimeEntryId,
    taskId,
    taskLabel: known.label,
    taskTitle: known.title,
    projectLabel: known.projectLabel,
    anchor: known.anchor,
    startedAt: started.toISOString(),
    stoppedAt: open ? null : stopped.toISOString(),
    createdAt: started.toISOString(),
    updatedAt: started.toISOString()
  }
}

// The week of Monday 31 August 2026.
const monday = new Date(2026, 7, 31)
const wednesday = new Date(2026, 8, 2)
const range = periodRange('week', wednesday)
const now = new Date(2026, 8, 2, 12, 0).getTime()

describe('buildTimeReport', () => {
  it('groups a week by company, then project, then task, biggest first', () => {
    const report = buildTimeReport(
      [
        session(builder, monday, 2),
        session(builder, wednesday, 1),
        session(retry, wednesday, 4),
        session(signal, monday, 3)
      ],
      tasks,
      companies,
      range,
      now
    )

    expect(report.totalSeconds).toBe(10 * 3600)
    expect(report.companies.map((company) => company.name)).toEqual(['acme', 'globex'])

    const [first] = report.companies
    expect(first!.totalSeconds).toBe(7 * 3600)
    expect(first!.share).toBeCloseTo(0.7)
    expect(first!.projects[0]!.tasks.map((row) => row.title)).toEqual([
      'Retry policy',
      'Report builder'
    ])
  })

  it('counts a session on the day it started, and spreads a task across the days it ran', () => {
    const report = buildTimeReport(
      [session(builder, monday, 2), session(builder, wednesday, 1)],
      tasks,
      companies,
      range,
      now
    )

    const row = report.companies[0]!.projects[0]!.tasks[0]!
    expect(row.totalSeconds).toBe(3 * 3600)
    expect(row.perDaySeconds[0]).toBe(2 * 3600)
    expect(row.perDaySeconds[2]).toBe(3600)
    expect(row.perDaySeconds[1]).toBe(0)
    expect(report.days[0]!.totalSeconds).toBe(2 * 3600)
    expect(report.busiestDay?.totalSeconds).toBe(2 * 3600)
  })

  it('counts a session still running up to now, and says that it is', () => {
    // Opened at 09:00, read at 12:00.
    const report = buildTimeReport([session(builder, wednesday, 0, true)], tasks, companies, range, now)

    const row = report.companies[0]!.projects[0]!.tasks[0]!
    expect(row.totalSeconds).toBe(3 * 3600)
    expect(row.isRunning).toBe(true)
  })

  it('leaves out what falls outside the period', () => {
    const lastWeek = new Date(2026, 7, 24)

    const report = buildTimeReport([session(builder, lastWeek, 5)], tasks, companies, range, now)

    expect(report.totalSeconds).toBe(0)
    expect(report.companies).toEqual([])
  })

  it('keeps only the companies picked, and reads no pick as all of them', () => {
    const entries = [session(builder, monday, 2), session(signal, monday, 3)]

    const everything = buildTimeReport(entries, tasks, companies, range, now)
    const onlyGlobex = buildTimeReport(entries, tasks, companies, range, now, new Set([globex]))

    expect(everything.companies).toHaveLength(2)
    expect(onlyGlobex.companies.map((company) => company.name)).toEqual(['globex'])
    expect(onlyGlobex.totalSeconds).toBe(3 * 3600)
    // The share is of what is shown, so a filtered report still adds up to a whole.
    expect(onlyGlobex.companies[0]!.share).toBe(1)
  })
})

describe('reportAsMarkdown', () => {
  it('carries the anchor of every task, so a line read back can be loaded again', () => {
    const report = buildTimeReport([session(builder, monday, 2)], tasks, companies, range, now)

    const markdown = reportAsMarkdown(report, range)

    expect(markdown.startsWith(`# Week of ${range.label}`)).toBe(true)
    expect(markdown).toContain('## acme · 2h')
    expect(markdown).toContain('- **2h** Report builder · `project:vega task:report-builder`')
  })

  it('says so plainly when there is nothing in the period', () => {
    const report = buildTimeReport([], tasks, companies, range, now)

    expect(reportAsMarkdown(report, range)).toContain('Nothing was tracked in this period.')
  })
})
