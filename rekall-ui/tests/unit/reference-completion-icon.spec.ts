import { describe, expect, it } from 'vitest'
import { referenceCompletionType, renderReferenceIcon } from '@/common/markdown/reference-completion-icon'

/** Each `[[` suggestion row starts with the glyph of what it names. */
describe('renderReferenceIcon', () => {
  it('draws the glyph named by the completion type', () => {
    const icon = renderReferenceIcon({ label: 'connection:prod', type: referenceCompletionType('KUBERNETES') }) as HTMLElement

    expect(icon.className).toBe('rk-completion-icon')
    expect(icon.dataset['icon']).toBe('kubernetes')
    expect(icon.querySelector('svg path')?.getAttribute('fill')).toBe('currentColor')
  })

  it('draws the console marks as outlines', () => {
    const icon = renderReferenceIcon({ label: 'task:export', type: referenceCompletionType('task') }) as HTMLElement

    expect(icon.dataset['icon']).toBe('task')
    expect(icon.querySelector('svg path')?.getAttribute('stroke')).toBe('currentColor')
  })

  it('leaves the editor’s other suggestions without one', () => {
    expect(renderReferenceIcon({ label: 'Heading', type: 'text' })).toBeNull()
  })
})
