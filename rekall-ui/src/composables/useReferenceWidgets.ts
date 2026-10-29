import { nextTick, onMounted, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { referenceTargetsChanged } from '@/common/live-preview/reference-targets'
import type { ReferenceTargets } from '@/common/markdown/reference-chip'

/** Keeps the editor's `[[...]]` chips true to the project's records: sent once the editor exists and again whenever they change. */
export function useReferenceWidgets(editor: Ref<ExposeParam | null>, targets: () => ReferenceTargets): void {
  function send(): void {
    void nextTick(() => editor.value?.getEditorView()?.dispatch({ effects: referenceTargetsChanged.of(targets()) }))
  }

  onMounted(send)
  watch(() => [targets().components, targets().connections, targets().tasks], send)
}
