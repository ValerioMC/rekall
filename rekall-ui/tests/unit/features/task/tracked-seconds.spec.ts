import { describe, expect, it } from 'vitest'
import { trackedSeconds } from '@/features/task/lib/tracked-seconds'
import type { TimeEntry } from '@/features/time/model/time-entry'
import { asTaskId, type TimeEntryId } from '@/model/branded'

function entry(startedAt: string, stoppedAt: string | null): TimeEntry {
  return {
    id: 'e' as TimeEntryId,
    taskId: asTaskId('t-1'),
    taskCode: 'RK-1',
    taskTitle: 'A task',
    projectLabel: 'rk',
    anchor: 'task:RK-1',
    startedAt,
    stoppedAt,
    createdAt: startedAt,
    updatedAt: startedAt
  }
}

describe('trackedSeconds', () => {
  it('adds the closed entries', () => {
    const entries = [
      entry('2026-11-13T09:00:00Z', '2026-11-13T09:30:00Z'),
      entry('2026-11-13T10:00:00Z', '2026-11-13T10:10:00Z')
    ]

    expect(trackedSeconds(entries, Date.parse('2026-11-13T12:00:00Z'))).toBe(2400)
  })

  it('counts a running entry up to now', () => {
    const entries = [entry('2026-11-13T09:00:00Z', null)]

    expect(trackedSeconds(entries, Date.parse('2026-11-13T09:05:00Z'))).toBe(300)
  })

  it('is zero with no entries', () => {
    expect(trackedSeconds([], 0)).toBe(0)
  })
})
