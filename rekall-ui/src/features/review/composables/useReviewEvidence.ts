import { computed, watch, type Ref } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useCheckStream } from '@/features/check/composables/useCheckStream'
import { useChecksStore } from '@/features/check/store/checks.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { TaskId } from '@/model/branded'

/** Reads the check results and deviations of each task that joins the queue, so a claim says what was found before it is opened. */
export function useReviewEvidence(items: Ref<readonly ReviewItem[]>): void {
  const checks = useChecksStore()
  const deviations = useDeviationsStore()
  const { run } = useAsyncAction()
  const taskIds = computed<readonly TaskId[]>(() => [...new Set(items.value.map((item) => item.taskId))])

  watch(
    taskIds,
    (now, before = []) => {
      for (const taskId of now.filter((id) => !before.includes(id))) {
        void run(() => checks.loadResults(taskId))
        if (!deviations.isKnown(taskId)) void run(() => deviations.load(taskId))
      }
    },
    { immediate: true }
  )

  useCheckStream({ onResults: (event) => void checks.applyResultEvent(event.taskId) })
}
