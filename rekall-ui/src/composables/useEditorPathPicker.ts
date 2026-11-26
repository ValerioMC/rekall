import { ref, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { useEditorSelectionPicker } from '@/composables/useEditorSelectionPicker'
import { formatPathForMarkdown } from '@/common/path/path-query'

/** The path picker of a markdown editor, opened with ⌘⇧P or the toolbar's folder. */
export function useEditorPathPicker(
  editor: Ref<ExposeParam | null>,
  options: { readonly: () => boolean; pathRoot: () => string | null }
) {
  const picker = useEditorSelectionPicker(editor, options.readonly)
  /** The folder the picker was last left in, so a second path from the same place is one keystroke. */
  const start = ref<string | null>(null)

  watch(options.pathRoot, () => (start.value = null))

  function format(path: string, directory: boolean): string {
    const target = picker.opened()
    return formatPathForMarkdown(path, directory, target?.doc.sliceString(0, target.from) ?? '')
  }

  function insert(path: string, directory: boolean): void {
    const target = picker.opened()
    picker.close()
    if (target) picker.write(target, format(path, directory))
  }

  /** ⌘⇧P (or Ctrl⇧P) toggles the picker while writing. */
  function onKeydown(event: KeyboardEvent): void {
    if (options.readonly() || !(event.metaKey || event.ctrlKey) || !event.shiftKey || event.code !== 'KeyP') return
    event.preventDefault()
    event.stopPropagation()
    picker.toggle()
  }

  return { open: picker.open, start, toggle: picker.toggle, close: picker.close, locateCaret: picker.locateCaret, format, insert, onKeydown }
}
