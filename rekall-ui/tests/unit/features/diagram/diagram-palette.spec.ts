import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PAPER_TOKENS, paperTokenReader } from '@/features/diagram/lib/diagram-palette'
import { nativeThemeVariables } from '@/features/diagram/lib/mermaid/native-theme'
import { drawingOf } from '@/features/diagram/lib/static-diagram'
import { drawSource } from '@/features/diagram/lib/sheet-drawing'

vi.mock('@/features/diagram/lib/sheet-drawing', () => ({ drawSource: vi.fn(async () => ({ drawing: 'native', svg: '', bounds: { x: 0, y: 0, width: 1, height: 1 } })) }))

describe('the paper palette', () => {
  it('draws Mermaid on a white ground in light mode', () => {
    const variables = nativeThemeVariables(paperTokenReader, false)

    expect(variables['darkMode']).toBe(false)
    expect(variables['mainBkg']).toBe(PAPER_TOKENS['--color-surface-raised'])
    expect(variables['primaryTextColor']).toBe('#1f2328')
  })

  it('keeps every colour a hex Mermaid can read', () => {
    const colours = Object.entries(PAPER_TOKENS).filter(([token]) => token.startsWith('--color-'))

    expect(colours.every(([, value]) => /^#[0-9a-f]{6}$/.test(value))).toBe(true)
  })
})

describe('drawingOf', () => {
  beforeEach(() => vi.mocked(drawSource).mockClear())

  it('draws a source once per palette, so the screen drawing is never replaced by the paper one', async () => {
    const source = 'flowchart LR\n  cache --> palette'

    await drawingOf(source, 'console')
    await drawingOf(source, 'paper')
    await drawingOf(source, 'paper')
    await drawingOf(source)

    expect(vi.mocked(drawSource).mock.calls.map(([, palette]) => palette)).toEqual(['console', 'paper'])
  })
})
