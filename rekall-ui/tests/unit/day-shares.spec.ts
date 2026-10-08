import { describe, expect, it } from 'vitest'
import { FULL_DAY_SECONDS, dayLoad, projectShares, totalSeconds } from '@/common/calendar/day-shares'
import type { DaySummaryRow } from '@/common/calendar/day-summary'
import type { ProjectId, TaskId } from '@/model/branded'

function row(taskId: string, projectId: string | null, seconds: number): DaySummaryRow {
  return {
    taskId: taskId as TaskId,
    taskTitle: taskId,
    anchor: `task:${taskId}`,
    projectId: projectId as ProjectId | null,
    projectLabel: projectId ?? '',
    totalSeconds: seconds,
    isRunning: false
  }
}

describe('projectShares', () => {
  it('sums a day per project, largest first', () => {
    const shares = projectShares([row('a', 'vega', 600), row('b', 'beacon', 3000), row('c', 'vega', 900)])

    expect(shares).toEqual([
      { id: 'beacon', seconds: 3000 },
      { id: 'vega', seconds: 1500 }
    ])
  })

  it('keeps a task with no known project as a share of its own', () => {
    expect(projectShares([row('orphan', null, 60)])).toEqual([{ id: 'orphan', seconds: 60 }])
  })
})

describe('dayLoad', () => {
  it('is the share of a working day, capped at a full one', () => {
    expect(dayLoad(0)).toBe(0)
    expect(dayLoad(FULL_DAY_SECONDS / 2)).toBe(0.5)
    expect(dayLoad(FULL_DAY_SECONDS * 3)).toBe(1)
    expect(dayLoad(-5)).toBe(0)
  })
})

describe('totalSeconds', () => {
  it('adds every row of a day', () => {
    expect(totalSeconds([row('a', 'vega', 60), row('b', null, 40)])).toBe(100)
    expect(totalSeconds([])).toBe(0)
  })
})
