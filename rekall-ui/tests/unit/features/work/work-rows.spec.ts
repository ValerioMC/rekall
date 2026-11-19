import { describe, expect, it } from 'vitest'
import { workRows } from '@/features/work/lib/work-rows'
import { reviewQueue } from '@/features/catalog/model/review'
import { step, task } from './work-fixtures'

const noTokens = (): number => 0

describe('workRows', () => {
  it('lists the tasks newest first', () => {
    const rows = workRows([task('RK-2'), task('RK-10'), task('RK-1')], [], noTokens)

    expect(rows.map((row) => row.task.code)).toEqual(['RK-10', 'RK-2', 'RK-1'])
  })

  it('counts each claimed step and a task handed in whole as one thing to review', () => {
    const withClaims = task('RK-1', { stepCount: 3 })
    const handedIn = task('RK-2', { reviewActive: true, reviewState: 'CLAIMED' })
    const steps = [step('s1', 'RK-1', 'CLAIMED'), step('s2', 'RK-1', 'CLAIMED'), step('s3', 'RK-1', 'OPEN')]

    const rows = workRows([withClaims, handedIn], steps, noTokens)

    expect(rows.find((row) => row.task.code === 'RK-1')).toMatchObject({ claimedSteps: 2, needs: 2, kind: 'review' })
    expect(rows.find((row) => row.task.code === 'RK-2')).toMatchObject({ claimedSteps: 0, needs: 1, kind: 'review' })
  })

  it('totals to what the review queue counts', () => {
    const tasks = [task('RK-1', { stepCount: 2 }), task('RK-2', { reviewActive: true, reviewState: 'CLAIMED' }), task('RK-3')]
    const steps = [step('s1', 'RK-1', 'CLAIMED'), step('s2', 'RK-1', 'DONE')]

    const needs = workRows(tasks, steps, noTokens).reduce((sum, row) => sum + row.needs, 0)

    expect(needs).toBe(reviewQueue(tasks, steps).length)
  })

  it('marks a task running while a step runs, and running outranks awaiting review', () => {
    const rows = workRows(
      [task('RK-1', { stepCount: 2 })],
      [step('s1', 'RK-1', 'RUNNING'), step('s2', 'RK-1', 'CLAIMED')],
      noTokens
    )

    expect(rows[0]).toMatchObject({ running: true, kind: 'running', needs: 1 })
  })

  it('marks a task with no checklist running while its review runs', () => {
    const rows = workRows([task('RK-1', { reviewActive: true, reviewState: 'RUNNING' })], [], noTokens)

    expect(rows[0]).toMatchObject({ running: true, kind: 'running' })
  })

  it.each([
    ['DONE', 'filed'],
    ['TODO', 'todo'],
    ['BLOCKED', 'blocked'],
    ['BACKLOG', 'backlog'],
    ['IN_PROGRESS', 'progress']
  ] as const)('names a %s task with nothing waiting %s', (status, kind) => {
    expect(workRows([task('RK-1', { status })], [], noTokens)[0]!.kind).toBe(kind)
  })

  it('files a finished task even when work is still claimed on it', () => {
    const rows = workRows([task('RK-1', { status: 'DONE', stepCount: 1 })], [step('s1', 'RK-1', 'CLAIMED')], noTokens)

    expect(rows[0]!.kind).toBe('filed')
  })

  it('carries the tokens generated for each task', () => {
    const rows = workRows([task('RK-1'), task('RK-2')], [], (id) => (id === 'RK-2' ? 1200 : 0))

    expect(rows.map((row) => [row.task.code, row.tokens])).toEqual([['RK-2', 1200], ['RK-1', 0]])
  })
})
