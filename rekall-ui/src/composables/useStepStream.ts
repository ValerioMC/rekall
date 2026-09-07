import { onScopeDispose, ref, type Ref } from 'vue'
import { env } from '@/common/config/env'
import { StepStreamEventSchema } from '@/api/schemas/catalog.schema'
import type { TaskStep } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The console's live feed of checklist changes.
 *
 * One `text/event-stream` connection to `/api/steps/stream`, held open for as long as the
 * console is mounted. Every step a session moves over MCP, and every box ticked in another
 * window, arrives here as a `steps` event carrying the affected task's whole checklist, and is
 * handed to `onEvent` for the store to apply. `EventSource` reconnects on its own if the
 * connection drops, so there is no backoff to run here.
 *
 * A payload that does not parse is dropped rather than thrown: a stale window should keep
 * working off its last good state, not break on one malformed frame.
 */
export function useStepStream(
  onEvent: (taskId: TaskId, steps: TaskStep[]) => void
): { connected: Ref<boolean>; stop: () => void } {
  const connected = ref(false)
  let source: EventSource | null = null

  function stop(): void {
    source?.close()
    source = null
    connected.value = false
  }

  // Guarded for the test environment, where EventSource may be absent: the console still renders,
  // it just does not receive live updates.
  if (typeof EventSource !== 'undefined') {
    source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)

    source.addEventListener('open', () => {
      connected.value = true
    })

    source.addEventListener('error', () => {
      // EventSource flips to CONNECTING and retries by itself; reflect that it is not live now.
      connected.value = source?.readyState === EventSource.OPEN
    })

    source.addEventListener('steps', (event) => {
      // A malformed frame is dropped, not thrown: a stale window keeps working off its last
      // good state rather than breaking on one bad line.
      try {
        const parsed = StepStreamEventSchema.safeParse(
          JSON.parse((event as MessageEvent<string>).data)
        )
        if (parsed.success) {
          onEvent(parsed.data.taskId, parsed.data.steps)
        }
      } catch {
        // not JSON, or not the shape we expect
      }
    })
  }

  onScopeDispose(stop)

  return { connected, stop }
}
