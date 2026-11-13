import { describe, expect, it } from 'vitest'
import { ageLabel } from '@/features/work/lib/age-label'

const NOW = Date.parse('2026-11-13T12:00:00Z')

describe('ageLabel', () => {
  it.each([
    ['2026-11-13T11:59:30Z', 'now'],
    ['2026-11-13T11:48:00Z', '12m'],
    ['2026-11-13T09:00:00Z', '3h'],
    ['2026-11-11T12:00:00Z', '2d'],
    ['2026-10-09T12:00:00Z', '5w'],
    ['2026-07-01T12:00:00Z', '4mo'],
    ['2024-11-01T12:00:00Z', '2y']
  ])('reads %s as %s', (iso, label) => {
    expect(ageLabel(iso, NOW)).toBe(label)
  })

  it('reads a date it cannot parse as now', () => {
    expect(ageLabel('', NOW)).toBe('now')
  })
})
