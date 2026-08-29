import { describe, expect, it } from 'vitest'
import { dateKey, isSameDay, isSameMonth, monthGridDays } from '@/common/calendar/month-grid'

describe('monthGridDays', () => {
  it('always returns six full weeks', () => {
    expect(monthGridDays(2026, 7)).toHaveLength(42)
  })

  it('starts the grid on a Monday', () => {
    const [first] = monthGridDays(2026, 7)
    expect(first!.getDay()).toBe(1)
  })

  it('includes the trailing days of the previous month up to the 1st', () => {
    // August 2026 starts on a Saturday, so the grid opens with the last Monday of July.
    const grid = monthGridDays(2026, 7)
    expect(grid[0]).toEqual(new Date(2026, 6, 27))
    expect(grid[5]).toEqual(new Date(2026, 7, 1))
  })

  it('carries into the next month to fill out the final week', () => {
    const grid = monthGridDays(2026, 7)
    const last = grid[grid.length - 1]!
    expect(last.getMonth()).toBe(8)
  })

  it('covers a month that already starts on a Monday without adding a leading week', () => {
    // June 2026 starts on a Monday.
    const grid = monthGridDays(2026, 5)
    expect(grid[0]).toEqual(new Date(2026, 5, 1))
  })
})

describe('isSameDay', () => {
  it('is true for the same calendar date regardless of time', () => {
    expect(isSameDay(new Date(2026, 7, 29, 3), new Date(2026, 7, 29, 23))).toBe(true)
  })

  it('is false across a day boundary', () => {
    expect(isSameDay(new Date(2026, 7, 29), new Date(2026, 7, 30))).toBe(false)
  })
})

describe('isSameMonth', () => {
  it('matches the given year and month', () => {
    expect(isSameMonth(new Date(2026, 7, 15), 2026, 7)).toBe(true)
    expect(isSameMonth(new Date(2026, 6, 15), 2026, 7)).toBe(false)
  })
})

describe('dateKey', () => {
  it('pads month and day to two digits', () => {
    expect(dateKey(new Date(2026, 0, 5))).toBe('2026-01-05')
  })
})
