import { describe, expect, it } from 'vitest'
import { contextFillLevel, fillOf } from '@/features/context-fill/lib/context-fill-level'
import { fill, OTHER_STEP, STEP } from './fixtures'

describe('contextFillLevel', () => {
  it('is neutral below 60, warning from 60 and danger from 85', () => {
    expect(contextFillLevel(0)).toBe('neutral')
    expect(contextFillLevel(59)).toBe('neutral')
    expect(contextFillLevel(60)).toBe('warning')
    expect(contextFillLevel(84)).toBe('warning')
    expect(contextFillLevel(85)).toBe('danger')
    expect(contextFillLevel(100)).toBe('danger')
  })
})

describe('fillOf', () => {
  const fills = [fill(null, 40), fill(STEP, 70)]

  it('finds the fill of a step and the task\'s own fill, which has no step', () => {
    expect(fillOf(fills, STEP)?.percent).toBe(70)
    expect(fillOf(fills, null)?.percent).toBe(40)
  })

  it('has none for a step no turn ran on', () => {
    expect(fillOf(fills, OTHER_STEP)).toBeNull()
    expect(fillOf([], null)).toBeNull()
  })
})
