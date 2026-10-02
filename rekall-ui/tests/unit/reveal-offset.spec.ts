import { describe, expect, it } from 'vitest'
import { revealOffset } from '@/common/scroll/reveal-offset'

const viewport = { top: 100, bottom: 500 }

describe('revealOffset', () => {
  it('leaves a row that is fully visible where it is', () => {
    expect(revealOffset({ top: 150, bottom: 190 }, viewport)).toBeNull()
  })

  it('moves a row below the viewport up to its middle', () => {
    expect(revealOffset({ top: 700, bottom: 740 }, viewport)).toBe(420)
  })

  it('moves a row above the viewport down to its middle', () => {
    expect(revealOffset({ top: -50, bottom: -10 }, viewport)).toBe(-330)
  })

  it('moves a row that is only partly visible', () => {
    expect(revealOffset({ top: 480, bottom: 520 }, viewport)).toBe(200)
  })
})
