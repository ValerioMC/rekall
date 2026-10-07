import { computed, nextTick, onBeforeUnmount, onMounted, ref, type ComputedRef, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { fittedEditorHeight, pixelsOf } from '@/common/config/editor-height'

/**
 * The height a growing editor takes: its text's own height plus everything around it (toolbar,
 * footer, borders), never less than `height`. A fixed editor keeps `height`. Wrapping changes with
 * the column's width and a diagram draws after the text, so either resize refits.
 */
export function useFittedEditorHeight(
  editor: Ref<ExposeParam | null>,
  root: Ref<HTMLElement | null>,
  options: { grow: () => boolean; height: () => string }
): { editorHeight: ComputedRef<string>; refit: () => void } {
  const fitted = ref<number | null>(null)
  const editorHeight = computed(() => (options.grow() && fitted.value !== null ? `${fitted.value}px` : options.height()))

  function refit(): void {
    const view = options.grow() ? editor.value?.getEditorView() : null
    const frame = root.value?.querySelector<HTMLElement>('.md-editor')
    if (!view || !frame) return
    const chrome = frame.offsetHeight - view.scrollDOM.clientHeight
    fitted.value = fittedEditorHeight(view.contentHeight, chrome, pixelsOf(options.height(), 420))
  }

  let sizeWatcher: ResizeObserver | null = null
  onMounted(() => {
    if (!options.grow()) return
    void nextTick(refit)
    if (typeof ResizeObserver === 'undefined' || !root.value) return
    sizeWatcher = new ResizeObserver(refit)
    sizeWatcher.observe(root.value)
    const content = editor.value?.getEditorView()?.contentDOM
    if (content) sizeWatcher.observe(content)
  })
  onBeforeUnmount(() => sizeWatcher?.disconnect())

  return { editorHeight, refit: () => void nextTick(refit) }
}
