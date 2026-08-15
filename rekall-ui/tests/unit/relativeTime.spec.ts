import { describe, expect, it } from 'vitest'
import { relativeTime } from '@/common/format/relative-time'

/**
 * A wrapup is read to decide whether to trust it, and "3 days ago" answers that where an ISO
 * timestamp does not. The one thing worth testing is where the phrasing changes hands.
 */
describe('relativeTime', () => {
  const now = Date.parse('2026-08-15T12:00:00Z')

  it('says just now inside the first minute', () => {
    expect(relativeTime('2026-08-15T11:59:30Z', now)).toBe('just now')
  })

  it('counts minutes, then hours, then days', () => {
    expect(relativeTime('2026-08-15T11:20:00Z', now)).toBe('40 min ago')
    expect(relativeTime('2026-08-15T11:00:00Z', now)).toBe('an hour ago')
    expect(relativeTime('2026-08-15T04:00:00Z', now)).toBe('8 hours ago')
    expect(relativeTime('2026-08-14T11:00:00Z', now)).toBe('yesterday')
    expect(relativeTime('2026-08-10T12:00:00Z', now)).toBe('5 days ago')
  })

  /** Past a month the distance stops meaning anything and the date itself says more. */
  it('gives the date once the distance stops being useful', () => {
    expect(relativeTime('2026-01-04T12:00:00Z', now)).not.toContain('ago')
    expect(relativeTime('2026-01-04T12:00:00Z', now)).toContain('2026')
  })

  it('returns nothing for a value it cannot read', () => {
    expect(relativeTime('not a date', now)).toBe('')
  })
})
