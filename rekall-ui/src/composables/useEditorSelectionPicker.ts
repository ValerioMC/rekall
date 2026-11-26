import { ref, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import type { CaretBox } from '@/composables/useCaretPlacement'

type EditorView = NonNullable<ReturnType<ExposeParam['getEditorView']>>

/** The selection a picker was opened over, with the text it was part of. */
export interface EditorSelection {
  readonly view: EditorView
  readonly doc: EditorView['state']['doc']
  readonly from: number
  readonly to: number
}

/**
 * A picker hung over a markdown editor's selection. What it writes goes into exactly that range
 * through CodeMirror, so it is one undo step and the v-model hears about it the way it hears about
 * typing.
 */
export function useEditorSelectionPicker(editor: Ref<ExposeParam | null>, readonly: () => boolean) {
  const open = ref(false)
  let selection: EditorSelection | null = null

  function view(): EditorView | null {
    return editor.value?.getEditorView() ?? null
  }

  function show(): void {
    const current = view()
    if (!current || readonly()) return
    const { from, to } = current.state.selection.main
    selection = { view: current, doc: current.state.doc, from, to }
    open.value = true
  }

  function close(): void {
    open.value = false
    view()?.focus()
  }

  function toggle(): void {
    if (open.value) close()
    else show()
  }

  /** Where the text will land, held inside the writing area when that point has scrolled out of it. */
  function locateCaret(): CaretBox | null {
    const current = view()
    if (!current || !selection) return null
    const frame = current.scrollDOM.getBoundingClientRect()
    const coords = current.coordsAtPos(selection.from)
    if (!coords) return { left: frame.left + 18, top: frame.top, bottom: frame.top + 20 }
    const top = Math.min(Math.max(coords.top, frame.top), frame.bottom - 20)
    return { left: coords.left, top, bottom: top + (coords.bottom - coords.top) }
  }

  function opened(): EditorSelection | null {
    return selection
  }

  /** False once the editor is gone or holds another text than the one the selection was taken from. */
  function stillHolds(target: EditorSelection): boolean {
    return view() === target.view && target.view.state.doc === target.doc
  }

  function write(target: EditorSelection, text: string): void {
    target.view.dispatch({
      changes: { from: target.from, to: target.to, insert: text },
      selection: { anchor: target.from + text.length },
      scrollIntoView: true,
      userEvent: 'input.paste'
    })
  }

  return { open, toggle, close, locateCaret, opened, stillHolds, write }
}
