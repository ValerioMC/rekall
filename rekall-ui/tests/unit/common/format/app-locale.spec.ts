import { describe, expect, it } from 'vitest'
import { APP_LOCALE, formatDate } from '@/common/format/app-locale'
import { weekdayLabels } from '@/features/time/lib/month-grid'

describe('the application locale', () => {
  it('writes a date day first, with two digits for the day and the month', () => {
    expect(formatDate(new Date(2026, 10, 9))).toBe('09/11/2026')
  })

  it('names days and months in English whatever the system speaks', () => {
    expect(APP_LOCALE).toBe('en-GB')
    expect(weekdayLabels()).toEqual(['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'])
    expect(new Date(2026, 10, 9).toLocaleDateString(APP_LOCALE, { month: 'long' })).toBe('November')
  })

  it('groups thousands with a comma', () => {
    expect((1201).toLocaleString(APP_LOCALE)).toBe('1,201')
  })
})
