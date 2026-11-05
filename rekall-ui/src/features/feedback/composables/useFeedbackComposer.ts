import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { WrittenFeedbackKind } from '@/features/feedback/model/feedback'

/** The text being written under the description and what it is about, for the task in view. */
export function useFeedbackComposer(): {
  draft: Ref<string>
  kind: ComputedRef<WrittenFeedbackKind>
  chooseKind: (chosen: WrittenFeedbackKind) => void
  canSubmit: ComputedRef<boolean>
  submit: () => Promise<void>
  reset: () => void
} {
  const store = useConsoleStore()
  const { selectedTaskId, selectedTaskSteps } = storeToRefs(store)
  const draft = ref('')
  const chosen = ref<WrittenFeedbackKind | null>(null)

  const defaultKind = computed<WrittenFeedbackKind>(() => (selectedTaskSteps.value.length > 0 ? 'PLAN' : 'IMPLEMENTATION'))
  const kind = computed<WrittenFeedbackKind>(() => chosen.value ?? defaultKind.value)
  const canSubmit = computed(() => selectedTaskId.value !== null && draft.value.trim() !== '')

  function chooseKind(next: WrittenFeedbackKind): void {
    chosen.value = next
  }

  function reset(): void {
    draft.value = ''
    chosen.value = null
  }

  async function submit(): Promise<void> {
    const taskId = selectedTaskId.value
    if (taskId === null || !canSubmit.value) return
    await store.addFeedback(taskId, kind.value, draft.value)
    reset()
  }

  return { draft, kind, chooseKind, canSubmit, submit, reset }
}
