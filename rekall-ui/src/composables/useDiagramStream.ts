import { onScopeDispose } from 'vue'
import { env } from '@/common/config/env'
import { DiagramStreamEventSchema } from '@/api/schemas/diagram.schema'
import type { DiagramStreamEvent } from '@/model/diagram'

/**
 * The `diagram` frames of the console's event stream, for the diagram screen alone: a diagram
 * a session writes appears in the library without a reload.
 */
export function useDiagramStream(onDiagram: (event: DiagramStreamEvent) => void): void {
  if (typeof EventSource === 'undefined') return
  const source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
  source.addEventListener('diagram', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = DiagramStreamEventSchema.safeParse(payload)
    if (parsed.success) onDiagram(parsed.data)
  })
  onScopeDispose(() => source.close())
}
