import { describe, expect, it } from 'vitest'
import { periodRange, shiftAnchor } from '@/common/report/period'

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

  /**
   * A week that crosses a month has to name both, and one that does not must not repeat it.
   * Asserted by counting the month names rather than by matching a string: the label is
   * formatted in whatever locale is reading it, and the rule under test is not.
   */
  it('names the month once inside it, twice across it', () => {
    const inside = periodRange('week', new Date(2026, 8, 9)).label
    const across = periodRange('week', new Date(2026, 7, 31)).label

    expect(monthNamesIn(inside)).toHaveLength(1)
    expect(inside).toMatch(/7\D+13\D+2026/)
    expect(monthNamesIn(across)).toHaveLength(2)
    expect(across).toMatch(/31\D+6\D+2026/)
  })
})

/** Whatever the locale spells the months as, they are the only letters in a week's label. */
function monthNamesIn(label: string): string[] {
  return label.match(/\p{L}+/gu) ?? []
}

describe('shiftAnchor', () => {
  it('steps a week at a time, across a month boundary', () => {
    expect(shiftAnchor('week', new Date(2026, 8, 3), -1)).toEqual(new Date(2026, 7, 27))
  })

  it('steps a month at a time, and lands on its first day', () => {
    expect(shiftAnchor('month', new Date(2026, 0, 31), 1)).toEqual(new Date(2026, 1, 1))
  })
})
