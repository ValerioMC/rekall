import { describe, expect, it } from 'vitest'
import { chartGeometry, niceCeiling, shortDay, taskTrend, trendOf, utcToday } from '@/common/token/token-trend'
import type { TaskId } from '@/model/branded'
import type { TokenDay } from '@/model/token-usage'

const day = (taskId: string, on: string, outputTokens: number): TokenDay => ({
  taskId: taskId as TaskId,
  day: on,
  inputTokens: 0,
  outputTokens,
  cacheCreationTokens: 0,
  cacheReadTokens: 0
})

const T1 = 't1' as TaskId

describe('a task’s token trend', () => {
  it('is empty for a task that generated nothing', () => {
    expect(taskTrend([day('t2', '2026-10-01', 5)], T1, 'all', '2026-10-05')).toEqual([])
  })

  it('runs as a total, one point per day, flat across quiet days, up to today', () => {
    const days = [day('t1', '2026-10-01', 5), day('t1', '2026-10-03', 7), day('t2', '2026-10-02', 99)]

    const points = taskTrend(days, T1, 'all', '2026-10-04')

    expect(points.map((point) => [point.day, point.generated, point.cumulative])).toEqual([
      ['2026-09-30', 0, 0],
      ['2026-10-01', 5, 5],
      ['2026-10-02', 0, 5],
      ['2026-10-03', 7, 12],
      ['2026-10-04', 0, 12]
    ])
  })

  it('draws a line for a task worked on one day only', () => {
    const points = taskTrend([day('t1', '2026-10-05', 9)], T1, '30d', '2026-10-05')

    expect(points.map((point) => point.cumulative)).toEqual([0, 9])
  })

  it('starts a range with the total already reached, so the line does not drop to zero', () => {
    const days = [day('t1', '2026-08-01', 100), day('t1', '2026-10-04', 5)]

    const points = taskTrend(days, T1, '30d', '2026-10-05')

    expect(points).toHaveLength(30)
    expect(points[0]).toMatchObject({ day: '2026-09-06', cumulative: 100 })
    expect(points[points.length - 1]).toMatchObject({ day: '2026-10-05', cumulative: 105 })
  })

  it('reaches back as far as the range says and no further than the first day', () => {
    const days = [day('t1', '2026-09-30', 1)]

    expect(taskTrend(days, T1, '90d', '2026-10-05')[0]!.day).toBe('2026-09-29')
    expect(taskTrend([day('t1', '2026-01-01', 1)], T1, '90d', '2026-10-05')).toHaveLength(90)
  })

  it('keeps going to a day after today rather than dropping it', () => {
    const points = taskTrend([day('t1', '2026-10-07', 3)], T1, 'all', '2026-10-05')

    expect(points[points.length - 1]).toMatchObject({ day: '2026-10-07', cumulative: 3 })
  })
})

describe('chart helpers', () => {
  it('rounds a scale top up to 1, 2 or 5 times a power of ten', () => {
    expect([0, 1, 1.5, 3, 7, 130, 4_900].map(niceCeiling)).toEqual([1, 1, 2, 5, 10, 200, 5_000])
  })

  it('scales values into the box, the top of the scale at the top', () => {
    const box = { width: 100, height: 50, top: 10, right: 0, bottom: 10, left: 0 }

    const geometry = chartGeometry([0, 5, 10], 10, box)

    expect(geometry.xs).toEqual([0, 50, 100])
    expect(geometry.ys).toEqual([40, 25, 10])
    expect(geometry.line).toBe('M0.0 40.0 L50.0 25.0 L100.0 10.0')
    expect(geometry.area.endsWith('L100.0 40.0 L0.0 40.0 Z')).toBe(true)
  })

  it('draws nothing for no values', () => {
    expect(chartGeometry([], 1, { width: 10, height: 10, top: 0, right: 0, bottom: 0, left: 0 }).line).toBe('')
  })

  it('names a day the way the server cut it, in UTC', () => {
    expect(shortDay('2026-10-12')).toBe('Oct 12')
    expect(utcToday(new Date('2026-10-12T23:30:00Z'))).toBe('2026-10-12')
  })
})

describe('trendOf', () => {
  it('adds up several tasks on the same day and ignores the rest', () => {
    const days = [day('t1', '2026-10-04', 3), day('t2', '2026-10-04', 4), day('t3', '2026-10-04', 100)]

    const points = trendOf(days, new Set([T1, 't2' as TaskId]), 'all', '2026-10-04')

    expect(points.at(-1)).toMatchObject({ day: '2026-10-04', generated: 7, cumulative: 7 })
  })
})
