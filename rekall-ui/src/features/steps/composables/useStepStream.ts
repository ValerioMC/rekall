import { onScopeDispose, ref, type Ref } from 'vue'
import { isConsoleStreamOpen, subscribeToFrame } from '@/common/stream/consoleEventStream'
import {
  NoteStreamEventSchema,
  StepStreamEventSchema,
  TaskReviewEventSchema,
  WrapupStreamEventSchema
} from '@/features/catalog/api/schemas/catalog.schema'
import { DeviationStreamEventSchema } from '@/features/deviations/api/schemas/deviation.schema'
import { FeedbackStreamEventSchema } from '@/features/feedback/api/schemas/feedback.schema'
import { CommitReferenceStreamEventSchema } from '@/features/commits/api/schemas/commitReference.schema'
import { RunQueueSchema } from '@/features/queue/api/schemas/runQueue.schema'
import type { NoteStreamEvent } from '@/features/notes/model/document'
import type { TaskReview, TaskStep } from '@/features/steps/model/task-step'
import type { WrapupStreamEvent } from '@/features/wrapup/model/wrapup'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { RunQueue } from '@/features/queue/model/runQueue'
import type { TaskId } from '@/model/branded'

/**
 * One SSE connection carrying nine console feeds: `steps`, `task-review`, `wrapup`,
 * `commit-reference`, `run-queue`, `note`, `deviation`, `feedback` and `task-created`.
 */
export function useStepStream(
  onSteps: (taskId: TaskId, steps: TaskStep[]) => void,
  onReview?: (review: TaskReview) => void,
  onWrapup?: (event: WrapupStreamEvent) => void,
  onCommit?: (reference: CommitReference) => void,
  onRunQueue?: (queue: RunQueue) => void,
  onNote?: (event: NoteStreamEvent) => void,
  onTaskCreated?: () => void,
  onDeviation?: (taskId: TaskId) => void,
  onFeedback?: (taskId: TaskId) => void
): { connected: Ref<boolean>; stop: () => void } {
  const connected = ref(isConsoleStreamOpen())
  const unsubscribers: Array<() => void> = []

  function stop(): void {
    unsubscribers.splice(0).forEach((unsubscribe) => unsubscribe())
    connected.value = false
  }

  function listen(frameName: string, handler: (event: Event) => void): void {
    unsubscribers.push(subscribeToFrame(frameName, handler))
  }

  function listenToPayload<T>(
    frameName: string,
    schema: { safeParse: (value: unknown) => { success: true; data: T } | { success: false } },
    handler: (payload: T) => void
  ): void {
    listen(frameName, (event) => {
      try {
        const parsed = schema.safeParse(JSON.parse((event as MessageEvent<string>).data))
        if (parsed.success) handler(parsed.data)
      } catch {
      }
    })
  }

  listen('open', () => {
    connected.value = true
  })

  listen('error', () => {
    connected.value = isConsoleStreamOpen()
  })

  listenToPayload('steps', StepStreamEventSchema, (event) => onSteps(event.taskId, event.steps))
  listenToPayload('task-review', TaskReviewEventSchema, (event) => onReview?.(event.review))
  listenToPayload('wrapup', WrapupStreamEventSchema, (event) => onWrapup?.(event))
  listenToPayload('commit-reference', CommitReferenceStreamEventSchema, (event) => onCommit?.(event.reference))
  listenToPayload('run-queue', RunQueueSchema, (queue) => onRunQueue?.(queue))
  listen('task-created', () => onTaskCreated?.())
  listenToPayload('note', NoteStreamEventSchema, (event) => onNote?.(event))
  listenToPayload('deviation', DeviationStreamEventSchema, (event) => onDeviation?.(event.taskId))
  listenToPayload('feedback', FeedbackStreamEventSchema, (event) => onFeedback?.(event.taskId))

  onScopeDispose(stop)

  return { connected, stop }
}
