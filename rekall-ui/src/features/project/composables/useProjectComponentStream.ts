import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { ProjectComponentStreamEventSchema } from '@/features/project/api/schemas/project-component.schema'
import type { ProjectComponentStreamEvent } from '@/features/project/model/project-component'

/**
 * The `project-component` frames of the console's event stream, for the project page alone: a
 * component a session writes appears in the list while the session works.
 */
export function useProjectComponentStream(onChange: (event: ProjectComponentStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('project-component', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ProjectComponentStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
