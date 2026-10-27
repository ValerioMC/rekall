import { onUnmounted, ref, watch, type Ref, type WritableComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEditorMode } from '@/composables/useEditorMode'
import type { EditorMode } from '@/stores/editorMode.store'
import type { TaskId } from '@/model/branded'

const SAVE_DELAY_MS = 700

export interface DescriptionDraft {
  readonly draft: Ref<string>
  /** Off while an empty description shows its invitation to write instead of the editor. */
  readonly showEditor: Ref<boolean>
  readonly mode: WritableComputedRef<EditorMode>
  readonly scheduleSave: () => void
  readonly beginWriting: () => void
  readonly toggleMode: () => void
}

/**
 * The selected task's description as it is being written: saved a moment after the last keystroke,
 * and saved at once when another task comes into view or the pane closes.
 */
export function useDescriptionDraft(): DescriptionDraft {
  const store = useConsoleStore()
  const { selectedTask } = storeToRefs(store)
  const { run } = useAsyncAction()
  const { mode, openFor } = useEditorMode('read')

  const showEditor = ref(false)
  const draft = ref('')
  let saveTimer: ReturnType<typeof setTimeout> | null = null

  function flush(taskId: TaskId): void {
    if (!saveTimer) return
    clearTimeout(saveTimer)
    saveTimer = null
    void run(() => store.saveTaskDescription(taskId, draft.value))
  }

  function scheduleSave(): void {
    const task = selectedTask.value
    if (!task) return
    store.saveState = 'unsaved'
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => {
      saveTimer = null
      void run(() => store.saveTaskDescription(task.id, draft.value))
    }, SAVE_DELAY_MS)
  }

  watch(
    () => selectedTask.value?.id ?? null,
    (_id, previousId) => {
      if (previousId) flush(previousId)
      draft.value = selectedTask.value?.description ?? ''
      showEditor.value = draft.value.trim().length > 0
      openFor(draft.value)
    },
    { immediate: true }
  )

  watch(
    () => selectedTask.value?.description ?? '',
    (value) => {
      if (saveTimer || value === draft.value) return
      draft.value = value
      if (value.trim()) showEditor.value = true
    }
  )

  function beginWriting(): void {
    showEditor.value = true
    mode.value = 'write'
  }

  function toggleMode(): void {
    if (!showEditor.value) return
    mode.value = mode.value === 'write' ? 'read' : 'write'
  }

  onUnmounted(() => {
    if (selectedTask.value) flush(selectedTask.value.id)
  })

  return { draft, showEditor, mode, scheduleSave, beginWriting, toggleMode }
}
