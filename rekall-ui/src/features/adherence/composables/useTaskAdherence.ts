import { computed, watch, type ComputedRef, type Ref } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { PlanAdherence } from '@/features/adherence/model/adherence'
import { useAdherenceStore } from '@/features/adherence/store/adherence.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/** A task's adherence, read again whenever a step of its plan changes state or gains a pass. */
export function useTaskAdherence(taskId: Ref<TaskId>, steps: Ref<readonly TaskStep[]>): ComputedRef<PlanAdherence | null> {
  const store = useAdherenceStore()
  const { run } = useAsyncAction()
  const planned = computed(() => steps.value.filter((step) => step.state !== 'DRAFT'))
  const signature = computed(() => planned.value.map((step) => `${step.id}:${step.state}:${step.passes.length}`).join('|'))

  watch(
    () => [taskId.value, signature.value],
    () => void run(() => store.loadTask(taskId.value)),
    { immediate: true }
  )

  return computed(() => store.ofTask(taskId.value))
}
