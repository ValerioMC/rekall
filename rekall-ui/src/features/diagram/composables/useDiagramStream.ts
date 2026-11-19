import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { DiagramStreamEventSchema } from '@/features/diagram/api/schemas/diagram.schema'
import type { DiagramStreamEvent } from '@/features/diagram/model/diagram'

/**
 * The `diagram` frames of the console's event stream, for the diagram screen alone: a diagram
 * a session writes appears in the library without a reload.
 */
export function useDiagramStream(onDiagram: (event: DiagramStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('diagram', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = DiagramStreamEventSchema.safeParse(payload)
    if (parsed.success) onDiagram(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
