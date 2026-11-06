import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import {
  ProjectCheckStreamEventSchema,
  TaskCheckResultStreamEventSchema
} from '@/features/check/api/schemas/check.schema'
import type { ProjectCheckStreamEvent, TaskCheckResultStreamEvent } from '@/features/check/model/check'

interface CheckStreamHandlers {
  readonly onChecks?: (event: ProjectCheckStreamEvent) => void
  readonly onResults?: (event: TaskCheckResultStreamEvent) => void
}

/**
 * The `project-check` and `task-check-result` frames of the console's event stream: a check
 * edited elsewhere, and a result a session's claim has just produced, show without a reload.
 */
export function useCheckStream(handlers: CheckStreamHandlers): void {
  const unsubscribeChecks = subscribeToFrame('project-check', (event) => {
    const parsed = ProjectCheckStreamEventSchema.safeParse(payloadOf(event))
    if (parsed.success) handlers.onChecks?.(parsed.data)
  })
  const unsubscribeResults = subscribeToFrame('task-check-result', (event) => {
    const parsed = TaskCheckResultStreamEventSchema.safeParse(payloadOf(event))
    if (parsed.success) handlers.onResults?.(parsed.data)
  })
  onScopeDispose(() => {
    unsubscribeChecks()
    unsubscribeResults()
  })
}

function payloadOf(event: Event): unknown {
  try {
    return JSON.parse((event as MessageEvent<string>).data)
  } catch {
    return null
  }
}
