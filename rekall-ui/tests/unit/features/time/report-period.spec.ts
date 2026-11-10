import { describe, expect, it } from 'vitest'
import { periodRange, shiftAnchor } from '@/features/time/lib/period'

/**
 * The frame every number on the report page is counted inside. Off by one here and the hours
 * are right and land in the wrong week, which is the kind of wrong nobody catches by looking.
 */
describe('periodRange', () => {
  it('runs a week Monday to Sunday, whichever day it is asked about', () => {
    // A Thursday.
    const range = periodRange('week', new Date(2026, 8, 3, 15, 30))

    expect(range.start).toEqual(new Date(2026, 7, 31))
    expect(range.end).toEqual(new Date(2026, 8, 7))
    expect(range.days).toHaveLength(7)
    expect(range.days[0]).toEqual(new Date(2026, 7, 31))
    expect(range.days[6]).toEqual(new Date(2026, 8, 6))
  })

  it('keeps a Monday in its own week rather than the one before', () => {
    const range = periodRange('week', new Date(2026, 8, 7, 0, 0))

    expect(range.start).toEqual(new Date(2026, 8, 7))
  })

  it('runs a month from the first to the last day, whatever its length', () => {
    const february = periodRange('month', new Date(2028, 1, 17))

    expect(february.start).toEqual(new Date(2028, 1, 1))
    expect(february.days).toHaveLength(29)
    expect(february.label).toContain('2028')
  })

  it('writes a week as its first and last day, day first', () => {
    expect(periodRange('week', new Date(2026, 8, 9)).label).toBe('07/09/2026 – 13/09/2026')
    expect(periodRange('week', new Date(2026, 7, 31)).label).toBe('31/08/2026 – 06/09/2026')
  })
})

describe('shiftAnchor', () => {
  it('steps a week at a time, across a month boundary', () => {
    expect(shiftAnchor('week', new Date(2026, 8, 3), -1)).toEqual(new Date(2026, 7, 27))
  })

  it('steps a month at a time, and lands on its first day', () => {
    expect(shiftAnchor('month', new Date(2026, 0, 31), 1)).toEqual(new Date(2026, 1, 1))
  })
})
