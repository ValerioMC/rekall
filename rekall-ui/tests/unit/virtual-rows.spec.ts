import { describe, expect, it } from 'vitest'
import { visibleRange } from '@/composables/useVirtualRows'

describe('the visible range of a long list', () => {
  it('covers the rows crossing the viewport, widened by the overscan', () => {
    expect(visibleRange(400, 200, 40, 1000, 2)).toEqual({ start: 8, end: 17 })
  })

  it('stays inside the list at both ends', () => {
    expect(visibleRange(0, 200, 40, 1000, 5)).toEqual({ start: 0, end: 10 })
    expect(visibleRange(39_800, 400, 40, 1000, 5)).toEqual({ start: 990, end: 1000 })
  })
})
