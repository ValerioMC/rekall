import { describe, expect, it } from 'vitest'
import { workLanes } from '@/features/work/lib/work-lanes'
import { rowsOfView } from '@/features/work/lib/work-view-rules'
import { workRows } from '@/features/work/lib/work-rows'
import { step, task } from './work-fixtures'

function rows() {
  const tasks = [
    task('PR-1', { status: 'TODO' }),
    task('PR-2', { status: 'BACKLOG' }),
    task('PR-3', { stepCount: 2 }),
    task('PR-4', { stepCount: 1 }),
    task('PR-5', { status: 'BLOCKED' }),
    task('PR-6', { stepCount: 1 }),
    task('PR-7', { status: 'DONE' })
  ]
  const steps = [step('s1', 'PR-3', 'RUNNING'), step('s2', 'PR-4', 'CLAIMED')]
  return workRows(tasks, steps, () => 0)
}

const codes = (lane: ReturnType<typeof workLanes>[number]): string[] => lane.rows.map((row) => row.task.code)

describe('workLanes', () => {
  it('deals the tasks into To do, In progress, Awaiting your review and Filed, newest first', () => {
    const lanes = workLanes(rows())

    expect(lanes.map((lane) => lane.label)).toEqual(['To do', 'In progress', 'Awaiting your review', 'Filed'])
    expect(codes(lanes[0]!)).toEqual(['PR-2', 'PR-1'])
    expect(codes(lanes[1]!)).toEqual(['PR-6', 'PR-5', 'PR-3'])
    expect(codes(lanes[2]!)).toEqual(['PR-4'])
    expect(codes(lanes[3]!)).toEqual(['PR-7'])
  })

  it('holds in its live lanes exactly the tasks the ledger lists on Open', () => {
    const all = rows()

    const live = workLanes(all).filter((lane) => lane.id !== 'filed').flatMap((lane) => codes(lane))

    expect(live.sort()).toEqual(rowsOfView(all, 'open').map((row) => row.task.code).sort())
  })

  it('files in Filed what the ledger lists on Filed', () => {
    const all = rows()

    const filed = workLanes(all).find((lane) => lane.id === 'filed')!

    expect(codes(filed)).toEqual(rowsOfView(all, 'filed').map((row) => row.task.code))
  })
})
