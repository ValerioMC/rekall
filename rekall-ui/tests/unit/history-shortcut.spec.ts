import { describe, expect, it } from 'vitest'
import { historyDirectionOf, historyShortcutLabel, runsOnMac } from '@/common/shell/history-shortcut'

const chord = (code: string, held: Partial<Record<'metaKey' | 'altKey' | 'ctrlKey' | 'shiftKey', boolean>>) => ({
  code,
  metaKey: false,
  altKey: false,
  ctrlKey: false,
  shiftKey: false,
  ...held
})

describe('the history shortcut', () => {
  it('is Cmd+Option+Arrow on a Mac', () => {
    expect(historyDirectionOf(chord('ArrowLeft', { metaKey: true, altKey: true }), true)).toBe('back')
    expect(historyDirectionOf(chord('ArrowRight', { metaKey: true, altKey: true }), true)).toBe('forward')
  })

  it('is Alt+Arrow elsewhere', () => {
    expect(historyDirectionOf(chord('ArrowLeft', { altKey: true }), false)).toBe('back')
    expect(historyDirectionOf(chord('ArrowRight', { altKey: true }), false)).toBe('forward')
  })

  it('leaves word and line movement alone', () => {
    expect(historyDirectionOf(chord('ArrowLeft', { altKey: true }), true)).toBeNull()
    expect(historyDirectionOf(chord('ArrowLeft', { metaKey: true }), true)).toBeNull()
    expect(historyDirectionOf(chord('ArrowLeft', {}), false)).toBeNull()
  })

  it('refuses a chord with Shift or Ctrl added, and any other key', () => {
    expect(historyDirectionOf(chord('ArrowLeft', { metaKey: true, altKey: true, shiftKey: true }), true)).toBeNull()
    expect(historyDirectionOf(chord('ArrowLeft', { metaKey: true, altKey: true, ctrlKey: true }), true)).toBeNull()
    expect(historyDirectionOf(chord('ArrowUp', { metaKey: true, altKey: true }), true)).toBeNull()
  })

  it('names the chord for the platform', () => {
    expect(historyShortcutLabel('back', true)).toBe('⌘⌥←')
    expect(historyShortcutLabel('forward', false)).toBe('Alt+→')
    expect(runsOnMac('MacIntel')).toBe(true)
    expect(runsOnMac('Win32')).toBe(false)
  })
})
