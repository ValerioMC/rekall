import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useEditorMode } from '@/composables/useEditorMode'

/** A reader who left an editor in Write finds the next one in Write: the pick follows them around. */
describe('the editor mode', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it("opens in the editor's own mode until the reader picks one", () => {
    const description = useEditorMode('read')
    description.openFor('A brief.')

    expect(description.mode.value).toBe('read')
  })

  it('opens every later editor in the mode last picked on any of them', () => {
    useEditorMode('read').mode.value = 'write'

    const next = useEditorMode('read')
    next.openFor('Another brief.')

    expect(next.mode.value).toBe('write')
  })

  it('opens an empty body to write, without taking that as a pick', () => {
    const editor = useEditorMode('read')
    editor.openFor('   ')
    expect(editor.mode.value).toBe('write')

    editor.openFor('Now there is text.')
    expect(editor.mode.value).toBe('read')
  })
})
