import { describe, expect, it } from 'vitest'
import { summarizeByDay } from '@/common/calendar/day-summary'
import type { TimeEntry } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

function entry(overrides: Partial<TimeEntry>): TimeEntry {
  return {
    id: 'te1' as TimeEntry['id'],
    taskId: 't1' as TaskId,
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: 'project:vega task:report-builder',
    startedAt: '2026-08-29T09:00:00.000Z',
    stoppedAt: '2026-08-29T10:00:00.000Z',
    createdAt: '2026-08-29T09:00:00.000Z',
    updatedAt: '2026-08-29T10:00:00.000Z',
    ...overrides
  }
}

const NOW = Date.parse('2026-08-29T12:00:00.000Z')

describe('summarizeByDay', () => {
  it('buckets a closed session under the day it started on', () => {
    const byDay = summarizeByDay([entry({})], NOW)
    const rows = byDay.get('2026-08-29')
    expect(rows).toHaveLength(1)
    expect(rows![0]).toMatchObject({ taskId: 't1', totalSeconds: 3600, isRunning: false })
  })

  it('sums two sessions on the same task and day into one row', () => {
    const byDay = summarizeByDay(
      [
        entry({ id: 'te1' as TimeEntry['id'], startedAt: '2026-08-29T09:00:00.000Z', stoppedAt: '2026-08-29T09:30:00.000Z' }),
        entry({ id: 'te2' as TimeEntry['id'], startedAt: '2026-08-29T11:00:00.000Z', stoppedAt: '2026-08-29T11:15:00.000Z' })
      ],
      NOW
    )
    const rows = byDay.get('2026-08-29')!
    expect(rows).toHaveLength(1)
    expect(rows[0]!.totalSeconds).toBe(45 * 60)
  })

  it('keeps two different tasks on the same day as separate rows', () => {
    const byDay = summarizeByDay(
      [entry({ taskId: 't1' as TaskId }), entry({ id: 'te2' as TimeEntry['id'], taskId: 't2' as TaskId })],
      NOW
    )
    expect(byDay.get('2026-08-29')).toHaveLength(2)
  })

  it('counts an open session up to now, and marks it running', () => {
    const byDay = summarizeByDay([entry({ stoppedAt: null })], NOW)
    const row = byDay.get('2026-08-29')![0]!
    expect(row.isRunning).toBe(true)
    expect(row.totalSeconds).toBe(3 * 3600)
  })

  it('sorts a day\'s rows by total time, longest first', () => {
    const byDay = summarizeByDay(
      [
        entry({ taskId: 't1' as TaskId, startedAt: '2026-08-29T09:00:00.000Z', stoppedAt: '2026-08-29T09:10:00.000Z' }),
        entry({ id: 'te2' as TimeEntry['id'], taskId: 't2' as TaskId, startedAt: '2026-08-29T09:00:00.000Z', stoppedAt: '2026-08-29T10:00:00.000Z' })
      ],
      NOW
    )
    const rows = byDay.get('2026-08-29')!
    expect(rows.map((row) => row.taskId)).toEqual(['t2', 't1'])
  })
})
