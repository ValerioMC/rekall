import { ref, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import type { CaretBox } from '@/composables/useCaretPlacement'
import { formatPathForMarkdown } from '@/common/path/path-query'

type EditorView = NonNullable<ReturnType<ExposeParam['getEditorView']>>

/**
 * The path picker of a markdown editor. It opens over the selection as it stands (⌘⇧P, or the
 * toolbar's folder) and writes into exactly that range when a path is chosen, through CodeMirror,
 * so the insert is one undo step and the v-model hears about it the way it hears about typing.
 */
export function useEditorPathPicker(
  editor: Ref<ExposeParam | null>,
  options: { readonly: () => boolean; pathRoot: () => string | null }
) {
  const open = ref(false)
  /** The folder the picker was last left in, so a second path from the same place is one keystroke. */
  const start = ref<string | null>(null)
  let range: { from: number; to: number } | null = null
  let textBefore = ''

  watch(options.pathRoot, () => (start.value = null))

  function view(): EditorView | null {
    return editor.value?.getEditorView() ?? null
  }

  function show(): void {
    const current = view()
    if (!current || options.readonly()) return
    const { from, to } = current.state.selection.main
    range = { from, to }
    textBefore = current.state.sliceDoc(0, from)
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

  /** Where the path will land, held inside the writing area when that point has scrolled out of it. */
  function locateCaret(): CaretBox | null {
    const current = view()
    if (!current || !range) return null
    const frame = current.scrollDOM.getBoundingClientRect()
    const coords = current.coordsAtPos(range.from)
    if (!coords) return { left: frame.left + 18, top: frame.top, bottom: frame.top + 20 }
    const top = Math.min(Math.max(coords.top, frame.top), frame.bottom - 20)
    return { left: coords.left, top, bottom: top + (coords.bottom - coords.top) }
  }

  function format(path: string, directory: boolean): string {
    return formatPathForMarkdown(path, directory, textBefore)
  }

  function insert(path: string, directory: boolean): void {
    const current = view()
    const target = range
    close()
    if (!current || !target) return
    const text = format(path, directory)
    current.dispatch({
      changes: { from: target.from, to: target.to, insert: text },
      selection: { anchor: target.from + text.length },
      scrollIntoView: true,
      userEvent: 'input.paste'
    })
  }

  /** ⌘⇧P (or Ctrl⇧P) toggles the picker while writing. */
  function onKeydown(event: KeyboardEvent): void {
    if (options.readonly() || !(event.metaKey || event.ctrlKey) || !event.shiftKey || event.code !== 'KeyP') return
    event.preventDefault()
    event.stopPropagation()
    toggle()
  }

  return { open, start, toggle, close, locateCaret, format, insert, onKeydown }
}
