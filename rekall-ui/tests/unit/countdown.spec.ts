import { describe, expect, it } from 'vitest'
import { formatResetIn } from '@/common/format/countdown'

const NOW = Date.parse('2026-09-08T12:00:00Z')

describe('formatResetIn', () => {
  it('is empty for a missing or unparseable instant', () => {
    expect(formatResetIn(null, NOW)).toBe('')
    expect(formatResetIn('not a date', NOW)).toBe('')
  })

  it('collapses anything already elapsed to now, never a negative clock', () => {
    expect(formatResetIn('2026-09-08T11:59:59Z', NOW)).toBe('now')
    expect(formatResetIn('2026-09-01T00:00:00Z', NOW)).toBe('now')
  })

  it('says less than a minute rather than 0m', () => {
    expect(formatResetIn('2026-09-08T12:00:30Z', NOW)).toBe('< 1m')
  })

  it('shows hours and minutes inside a day', () => {
    expect(formatResetIn('2026-09-08T15:24:00Z', NOW)).toBe('3h 24m')
    expect(formatResetIn('2026-09-08T12:12:00Z', NOW)).toBe('12m')
  })

  it('drops to days and hours past a day out', () => {
    expect(formatResetIn('2026-09-10T18:00:00Z', NOW)).toBe('2d 6h')
  })
})
