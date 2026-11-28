import { describe, expect, it } from 'vitest'
import { matchesQuery, nextSort, rowsOfView, sortRows, viewCounts } from '@/features/project/lib/project-task-rules'
import type { ListedTask, ProjectTaskSortKey } from '@/features/project/model/project-task-list'
import { listed, step, task } from './project-task-fixtures'

function rows(tokens: Record<string, number> = {}): ListedTask[] {
  const tasks = [
    task('RK-1', { title: 'Plan the launch', stepCount: 2, updatedAt: '2026-09-01T10:00:00Z' }),
    task('RK-2', { title: 'Write the docs', status: 'TODO', updatedAt: '2026-09-05T10:00:00Z', documentCount: 3, tagName: 'docs' }),
    task('RK-3', { title: 'Ship the build', status: 'DONE', updatedAt: '2026-09-03T10:00:00Z', stepCount: 1, stepsDone: 1 }),
    task('RK-4', { title: 'Fix the login', stepCount: 1, updatedAt: '2026-09-04T10:00:00Z', tagName: 'bug' }),
    task('RK-5', { title: 'Archive the kickoff', status: 'DONE', updatedAt: '2026-08-01T10:00:00Z' })
  ]
  const steps = [step('s1', 'RK-1', 'CLAIMED'), step('s2', 'RK-1', 'CLAIMED'), step('s3', 'RK-4', 'RUNNING')]
  return listed(tasks, steps, tokens)
}

const codes = (sorted: readonly ListedTask[]): string[] => sorted.map((row) => row.task.code)

describe('rowsOfView', () => {
  it('lists what needs a review in Needs me', () => {
    expect(codes(rowsOfView(rows(), 'needs'))).toEqual(['RK-1'])
  })

  it('lists the tasks a session works on in Running', () => {
    expect(codes(rowsOfView(rows(), 'running'))).toEqual(['RK-4'])
  })

  it('lists everything not filed in Open', () => {
    expect(codes(rowsOfView(rows(), 'open'))).toEqual(['RK-4', 'RK-2', 'RK-1'])
  })

  it('lists the finished tasks in Filed', () => {
    expect(codes(rowsOfView(rows(), 'filed'))).toEqual(['RK-5', 'RK-3'])
  })

  it('lists every task in All', () => {
    expect(rowsOfView(rows(), 'all')).toHaveLength(5)
  })
})

describe('viewCounts', () => {
  it('counts review items in Needs me and tasks in the other tabs', () => {
    expect(viewCounts(rows())).toEqual({ needs: 2, running: 1, open: 3, filed: 2, all: 5 })
  })
})

describe('matchesQuery', () => {
  const claimed = (): ListedTask => rowsOfView(rows(), 'needs')[0]!

  it('matches a title or a code, in any case', () => {
    expect(matchesQuery(claimed(), 'LAUNCH')).toBe(true)
    expect(matchesQuery(claimed(), 'rk-1')).toBe(true)
  })

  it('matches the text of the step line', () => {
    expect(matchesQuery(claimed(), 'step s1')).toBe(true)
  })

  it('needs every word of the query to appear', () => {
    expect(matchesQuery(claimed(), 'plan launch')).toBe(true)
    expect(matchesQuery(claimed(), 'plan docs')).toBe(false)
  })

  it('matches everything on a blank query', () => {
    expect(matchesQuery(claimed(), '   ')).toBe(true)
  })
})

describe('sortRows', () => {
  const by = (key: ProjectTaskSortKey, descending: boolean, tokens: Record<string, number> = {}): string[] =>
    codes(sortRows(rows(tokens), { key, descending }))

  it('orders by task number both ways', () => {
    expect(by('task', true)).toEqual(['RK-5', 'RK-4', 'RK-3', 'RK-2', 'RK-1'])
    expect(by('task', false)).toEqual(['RK-1', 'RK-2', 'RK-3', 'RK-4', 'RK-5'])
  })

  it('orders by title', () => {
    expect(by('title', false)[0]).toBe('RK-5')
  })

  it('orders by tag, the untagged first from A, newest first among equals', () => {
    expect(by('tag', false)).toEqual(['RK-5', 'RK-3', 'RK-1', 'RK-4', 'RK-2'])
  })

  it('puts a running session first by status, then what awaits review', () => {
    expect(by('status', true).slice(0, 2)).toEqual(['RK-4', 'RK-1'])
  })

  it('orders by the share of the plan handed in, tasks without steps last', () => {
    expect(by('plan', true).slice(0, 2)).toEqual(['RK-3', 'RK-1'])
  })

  it('orders by notes', () => {
    expect(by('notes', true)[0]).toBe('RK-2')
  })

  it('orders by tokens, the most first', () => {
    expect(by('tokens', true, { 'RK-2': 50, 'RK-3': 900 }).slice(0, 2)).toEqual(['RK-3', 'RK-2'])
  })

  it('orders by time tracked', () => {
    const timed = rows().map((row) => ({ ...row, trackedSeconds: row.task.code === 'RK-2' ? 60 : 0 }))

    expect(codes(sortRows(timed, { key: 'time', descending: true }))[0]).toBe('RK-2')
  })

  it('orders by last update, the latest first', () => {
    expect(by('updated', true).slice(0, 3)).toEqual(['RK-2', 'RK-4', 'RK-3'])
  })
})

describe('nextSort', () => {
  it('turns the key in use round', () => {
    expect(nextSort({ key: 'tokens', descending: true }, 'tokens')).toEqual({ key: 'tokens', descending: false })
  })

  it('starts a new key from A for text and from the most otherwise', () => {
    expect(nextSort({ key: 'task', descending: true }, 'title')).toEqual({ key: 'title', descending: false })
    expect(nextSort({ key: 'task', descending: false }, 'updated')).toEqual({ key: 'updated', descending: true })
  })
})
