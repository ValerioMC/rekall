import { describe, expect, it } from 'vitest'
import { formatClock, formatDuration } from '@/common/format/duration'

describe('formatClock', () => {
  it('pads every segment to two digits', () => {
    expect(formatClock(0)).toBe('00:00:00')
    expect(formatClock(5)).toBe('00:00:05')
    expect(formatClock(65)).toBe('00:01:05')
  })

  it('carries minutes into hours', () => {
    expect(formatClock(3600)).toBe('01:00:00')
    expect(formatClock(3661)).toBe('01:01:01')
  })

  it('does not go negative on a clock that has not started counting up yet', () => {
    expect(formatClock(-4)).toBe('00:00:00')
  })
})

describe('formatDuration', () => {
  it('says zero plainly, rather than implying a session that rounded away', () => {
    expect(formatDuration(0)).toBe('0m')
  })

  it('says less than a minute for a session that happened but rounds down to nothing', () => {
    expect(formatDuration(1)).toBe('< 1m')
    expect(formatDuration(59)).toBe('< 1m')
  })

  it('drops the hour once there is none', () => {
    expect(formatDuration(60)).toBe('1m')
    expect(formatDuration(45 * 60)).toBe('45m')
  })

  it('drops the minutes once there are none', () => {
    expect(formatDuration(3600)).toBe('1h')
    expect(formatDuration(2 * 3600)).toBe('2h')
  })

  it('shows both once there are both', () => {
    expect(formatDuration(6 * 3600 + 42 * 60)).toBe('6h 42m')
  })
})
