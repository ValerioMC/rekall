import { onScopeDispose, ref, type Ref } from 'vue'
import { env } from '@/common/config/env'
import {
  NoteStreamEventSchema,
  StepStreamEventSchema,
  TaskReviewEventSchema,
  WrapupStreamEventSchema
} from '@/api/schemas/catalog.schema'
import { CommitReferenceStreamEventSchema } from '@/api/schemas/commitReference.schema'
import { RunQueueSchema } from '@/api/schemas/runQueue.schema'
import type { NoteStreamEvent, TaskReview, TaskStep, WrapupStreamEvent } from '@/model/catalog'
import type { CommitReference } from '@/model/commitReference'
import type { RunQueue } from '@/model/runQueue'
import type { TaskId } from '@/model/branded'

/**
 * One SSE connection carrying six console feeds: `steps`, `task-review`, `wrapup`,
 * `commit-reference`, `run-queue` and `note`.
 */
export function useStepStream(
  onSteps: (taskId: TaskId, steps: TaskStep[]) => void,
  onReview?: (review: TaskReview) => void,
  onWrapup?: (event: WrapupStreamEvent) => void,
  onCommit?: (reference: CommitReference) => void,
  onRunQueue?: (queue: RunQueue) => void,
  onNote?: (event: NoteStreamEvent) => void
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

    source.addEventListener('wrapup', (event) => {
      try {
        const parsed = WrapupStreamEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onWrapup?.(parsed.data)
        }
      } catch {
      }
    })

    // A commit logged from a session, by hand or by auto-commit, lands in the rail without a reload.
    source.addEventListener('commit-reference', (event) => {
      try {
        const parsed = CommitReferenceStreamEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onCommit?.(parsed.data.reference)
        }
      } catch {
      }
    })

    // The run queue moves on the server's clock, so the beacon learns of a start, a hold or a
    // finished task from here rather than by asking.
    source.addEventListener('run-queue', (event) => {
      try {
        const parsed = RunQueueSchema.safeParse(JSON.parse((event as MessageEvent<string>).data))
        if (parsed.success) {
          onRunQueue?.(parsed.data)
        }
      } catch {
      }
    })

    // A note a session wrote onto a task shows up in its list without a reload.
    source.addEventListener('note', (event) => {
      try {
        const parsed = NoteStreamEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onNote?.(parsed.data)
        }
      } catch {
      }
    })
  }

  onScopeDispose(stop)

  return { connected, stop }
}
