import { describe, expect, it } from 'vitest'
import { workRows } from '@/features/work/lib/work-rows'
import { groupByProject, matchesQuery, rowsOfView, sortRows, viewCounts } from '@/features/work/lib/work-view-rules'
import type { WorkRow } from '@/features/work/model/work-view'
import type { ProjectId } from '@/model/branded'
import { project, step, task } from './work-fixtures'

const vega = 'p1' as ProjectId
const orion = 'p2' as ProjectId

function rows(tokens: Record<string, number> = {}): WorkRow[] {
  const tasks = [
    task('RK-1', { title: 'Plan the launch', stepCount: 2, updatedAt: '2026-09-01T10:00:00Z' }),
    task('RK-2', { title: 'Write the docs', status: 'TODO', updatedAt: '2026-09-05T10:00:00Z' }),
    task('RK-3', { title: 'Ship the build', status: 'DONE', updatedAt: '2026-09-03T10:00:00Z' }),
    task('RK-4', { title: 'Fix the login', stepCount: 1, updatedAt: '2026-09-04T10:00:00Z' }),
    task('RK-5', { title: 'Orion kickoff', projectId: orion, projectTitle: 'Orion', status: 'DONE', updatedAt: '2026-08-01T10:00:00Z' })
  ]
  const steps = [step('s1', 'RK-1', 'CLAIMED'), step('s2', 'RK-1', 'CLAIMED'), step('s3', 'RK-4', 'RUNNING')]
  return workRows(tasks, steps, (id) => tokens[id] ?? 0)
}

describe('rowsOfView', () => {
  it('lists what needs a review in Needs me', () => {
    expect(rowsOfView(rows(), 'needs').map((row) => row.task.code)).toEqual(['RK-1'])
  })

  it('lists the tasks a session works on in Running', () => {
    expect(rowsOfView(rows(), 'running').map((row) => row.task.code)).toEqual(['RK-4'])
  })

  it('lists everything not filed in Open', () => {
    expect(rowsOfView(rows(), 'open').map((row) => row.task.code)).toEqual(['RK-4', 'RK-2', 'RK-1'])
  })

  it('lists the finished tasks in Filed', () => {
    expect(rowsOfView(rows(), 'filed').map((row) => row.task.code)).toEqual(['RK-5', 'RK-3'])
  })
})

describe('viewCounts', () => {
  it('counts review items in Needs me and tasks in the other tabs', () => {
    expect(viewCounts(rows())).toEqual({ needs: 2, running: 1, open: 3, filed: 2 })
  })
})

describe('matchesQuery', () => {
  it('matches a title or a code, in any case', () => {
    const [first] = rowsOfView(rows(), 'needs')

    expect(matchesQuery(first!, 'LAUNCH')).toBe(true)
    expect(matchesQuery(first!, 'rk-1')).toBe(true)
  })

  it('needs every word of the query to appear', () => {
    const [first] = rowsOfView(rows(), 'needs')

    expect(matchesQuery(first!, 'plan launch')).toBe(true)
    expect(matchesQuery(first!, 'plan docs')).toBe(false)
  })

  it('matches everything on a blank query', () => {
    const [first] = rowsOfView(rows(), 'needs')

    expect(matchesQuery(first!, '   ')).toBe(true)
  })
})

describe('sortRows', () => {
  const codes = (sorted: readonly WorkRow[]): string[] => sorted.map((row) => row.task.code)

  it('sorts by task, newest first', () => {
    expect(codes(sortRows(rows(), 'task'))).toEqual(['RK-5', 'RK-4', 'RK-3', 'RK-2', 'RK-1'])
  })

  it('sorts by what needs you, the most first, newest first among equals', () => {
    expect(codes(sortRows(rows(), 'needs'))).toEqual(['RK-1', 'RK-5', 'RK-4', 'RK-3', 'RK-2'])
  })

  it('sorts by tokens, the most first', () => {
    expect(codes(sortRows(rows({ 'RK-2': 50, 'RK-3': 900 }), 'tokens')).slice(0, 2)).toEqual(['RK-3', 'RK-2'])
  })

  it('sorts by last update, the latest first', () => {
    expect(codes(sortRows(rows(), 'updated')).slice(0, 3)).toEqual(['RK-2', 'RK-4', 'RK-3'])
  })
})

describe('groupByProject', () => {
  const projects = [project('p1', 'Vega'), project('p2', 'Orion')]

  it('gathers the shown rows under their projects and folds the finished ones', () => {
    const all = rows()

    const groups = groupByProject(rowsOfView(all, 'open'), rowsOfView(all, 'filed'), projects, false)

    expect(groups.map((group) => [group.projectId, group.rows.length, group.filed.length])).toEqual([[vega, 3, 1]])
  })

  it('leaves out a project with nothing shown, however many tasks it has filed', () => {
    const all = rows()

    const groups = groupByProject(rowsOfView(all, 'open'), rowsOfView(all, 'filed'), projects, false)

    expect(groups.map((group) => group.projectId)).not.toContain(orion)
  })

  it('lists the finished tasks as rows when nothing is folded', () => {
    const groups = groupByProject(rowsOfView(rows(), 'filed'), [], projects, false)

    expect(groups.map((group) => [group.projectId, group.rows.length, group.filed.length])).toEqual([
      [orion, 1, 0],
      [vega, 1, 0]
    ])
  })

  it('totals what needs the reader and the tokens of the whole project', () => {
    const all = rows({ 'RK-1': 100, 'RK-3': 20 })

    const [group] = groupByProject(rowsOfView(all, 'needs'), rowsOfView(all, 'filed'), projects, false)

    expect(group).toMatchObject({ needs: 2, tokens: 120 })
  })

  it('puts a paused project after an active one', () => {
    const paused = [project('p1', 'Vega', { status: 'PAUSED' }), project('p2', 'Orion')]
    const all = rows()

    const groups = groupByProject(sortRows(all, 'task'), [], paused, false)

    expect(groups.map((group) => group.projectId)).toEqual([orion, vega])
  })
})
