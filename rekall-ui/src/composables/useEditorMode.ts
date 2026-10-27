import { computed, ref, type WritableComputedRef } from 'vue'
import { useEditorModeStore, type EditorMode } from '@/stores/editorMode.store'

export interface EditorModeControl {
  /** Setting it is the reader's pick, and every editor opened after it opens the same way. */
  readonly mode: WritableComputedRef<EditorMode>
  /** Puts the editor in the mode it opens in for this body, without counting as a pick. */
  readonly openFor: (body: string) => void
}

/** One markdown editor's Write/Read mode, opened as the reader last left any editor. */
export function useEditorMode(fallback: EditorMode): EditorModeControl {
  const preference = useEditorModeStore()
  const current = ref<EditorMode>(preference.picked ?? fallback)

  const mode = computed<EditorMode>({
    get: () => current.value,
    set: (next) => {
      current.value = next
      preference.pick(next)
    }
  })

  function openFor(body: string): void {
    current.value = preference.openingModeFor(body, fallback)
  }

  return { mode, openFor }
}
