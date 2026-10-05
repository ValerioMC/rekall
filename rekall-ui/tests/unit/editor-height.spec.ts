import { describe, expect, it } from 'vitest'
import { fittedEditorHeight, pixelsOf } from '@/common/config/editor-height'

describe('pixelsOf', () => {
  it('reads a pixel length', () => {
    expect(pixelsOf('520px', 400)).toBe(520)
    expect(pixelsOf(' 12.5px ', 400)).toBe(12.5)
  })

  it('falls back for any other unit', () => {
    expect(pixelsOf('50vh', 400)).toBe(400)
    expect(pixelsOf('auto', 400)).toBe(400)
  })
})

describe('fittedEditorHeight', () => {
  it('holds the minimum for short text', () => {
    expect(fittedEditorHeight(120, 70, 520)).toBe(520)
  })

  it('grows with the text past the minimum', () => {
    expect(fittedEditorHeight(1800, 70, 520)).toBe(1870)
  })

  it('rounds a fractional measure up so the last line is never clipped', () => {
    expect(fittedEditorHeight(900.2, 70, 520)).toBe(971)
  })
})
