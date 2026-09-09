import { onScopeDispose, ref, type Ref } from 'vue'
import { env } from '@/common/config/env'
import { StepStreamEventSchema, TaskReviewEventSchema } from '@/api/schemas/catalog.schema'
import type { TaskReview, TaskStep } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * One SSE connection carrying two console feeds: `steps` for a task's checklist,
 * `task-review` for the review line of a task that has none.
 */
export function useStepStream(
  onSteps: (taskId: TaskId, steps: TaskStep[]) => void,
  onReview?: (review: TaskReview) => void
): { connected: Ref<boolean>; stop: () => void } {
  const connected = ref(false)
  let source: EventSource | null = null

  function stop(): void {
    source?.close()
    source = null
    connected.value = false
  }

  if (typeof EventSource !== 'undefined') {
    source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)

    source.addEventListener('open', () => {
      connected.value = true
    })

    source.addEventListener('error', () => {
      connected.value = source?.readyState === EventSource.OPEN
    })

    source.addEventListener('steps', (event) => {
      try {
        const parsed = StepStreamEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onSteps(parsed.data.taskId, parsed.data.steps)
        }
      } catch {
      }
    })

    source.addEventListener('task-review', (event) => {
      try {
        const parsed = TaskReviewEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onReview?.(parsed.data.review)
        }
      } catch {
      }
    })
  }

  onScopeDispose(stop)

  return { connected, stop }
}
