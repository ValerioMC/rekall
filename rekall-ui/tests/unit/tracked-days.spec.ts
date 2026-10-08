import { describe, expect, it } from 'vitest'
import { trackedLastDays } from '@/common/calendar/tracked-days'
import type { TimeEntry } from '@/model/catalog'

const NOW = new Date(2026, 9, 9, 15, 0).getTime()

function entry(started: Date, stopped: Date | null): TimeEntry {
  return {
    id: `te-${started.getTime()}` as TimeEntry['id'],
    taskId: 't1' as TimeEntry['taskId'],
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: 'project:vega task:report-builder',
    startedAt: started.toISOString(),
    stoppedAt: stopped?.toISOString() ?? null,
    createdAt: started.toISOString(),
    updatedAt: started.toISOString()
  }
}

describe('trackedLastDays', () => {
  it('returns one row per day, oldest first, with today last', () => {
    const days = trackedLastDays([], NOW)

    expect(days).toHaveLength(7)
    expect(days[0]?.key).toBe('2026-10-03')
    expect(days[6]?.key).toBe('2026-10-09')
    expect(days.map((day) => day.isToday)).toEqual([false, false, false, false, false, false, true])
  })

  it('sums the entries of each day and leaves a day without any at zero', () => {
    const days = trackedLastDays(
      [
        entry(new Date(2026, 9, 7, 9), new Date(2026, 9, 7, 10)),
        entry(new Date(2026, 9, 7, 14), new Date(2026, 9, 7, 14, 30))
      ],
      NOW
    )

    expect(days.find((day) => day.key === '2026-10-07')?.seconds).toBe(5400)
    expect(days.find((day) => day.key === '2026-10-08')?.seconds).toBe(0)
  })

  it('runs a running entry up to now', () => {
    const days = trackedLastDays([entry(new Date(2026, 9, 9, 14), null)], NOW)

    expect(days[6]?.seconds).toBe(3600)
  })

  it('ignores entries older than the window', () => {
    const days = trackedLastDays([entry(new Date(2026, 8, 1, 9), new Date(2026, 8, 1, 10))], NOW)

    expect(days.every((day) => day.seconds === 0)).toBe(true)
  })
})
