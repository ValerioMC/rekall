import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { WrittenFeedbackKind } from '@/features/feedback/model/feedback'

/**
 * The text being written under the description, what it is about, and the applied entry it adds
 * detail to when there is one, for the task in view.
 */
export function useFeedbackComposer(): {
  draft: Ref<string>
  kind: ComputedRef<WrittenFeedbackKind>
  followsSeq: Ref<number | null>
  followUp: (seq: number) => void
  dropFollowUp: () => void
  chooseKind: (chosen: WrittenFeedbackKind) => void
  canSubmit: ComputedRef<boolean>
  submit: () => Promise<void>
  reset: () => void
} {
  const store = useConsoleStore()
  const { selectedTaskId, selectedTaskSteps } = storeToRefs(store)
  const draft = ref('')
  const chosen = ref<WrittenFeedbackKind | null>(null)
  const followsSeq = ref<number | null>(null)

  const defaultKind = computed<WrittenFeedbackKind>(() => (selectedTaskSteps.value.length > 0 ? 'PLAN' : 'IMPLEMENTATION'))
  const kind = computed<WrittenFeedbackKind>(() => chosen.value ?? defaultKind.value)
  const canSubmit = computed(() => selectedTaskId.value !== null && draft.value.trim() !== '')

  function chooseKind(next: WrittenFeedbackKind): void {
    chosen.value = next
  }

  function followUp(seq: number): void {
    followsSeq.value = seq
  }

  function dropFollowUp(): void {
    followsSeq.value = null
  }

  function reset(): void {
    draft.value = ''
    chosen.value = null
    followsSeq.value = null
  }

  async function submit(): Promise<void> {
    const taskId = selectedTaskId.value
    if (taskId === null || !canSubmit.value) return
    await store.addFeedback(taskId, kind.value, draft.value, followsSeq.value)
    reset()
  }

  return { draft, kind, followsSeq, followUp, dropFollowUp, chooseKind, canSubmit, submit, reset }
}
