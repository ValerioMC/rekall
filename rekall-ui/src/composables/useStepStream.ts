import { onScopeDispose, ref, type Ref } from 'vue'
import { env } from '@/common/config/env'
import { StepStreamEventSchema } from '@/api/schemas/catalog.schema'
import type { TaskStep } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

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
          onEvent(parsed.data.taskId, parsed.data.steps)
        }
      } catch {
      }
    })
  }

  onScopeDispose(stop)

  return { connected, stop }
}
