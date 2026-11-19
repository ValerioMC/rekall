import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { ProjectSectionStreamEventSchema } from '@/features/project/api/schemas/project-section.schema'
import type { ProjectSectionStreamEvent } from '@/features/project/model/project-section'

/**
 * The `project-section` frames of the console's event stream, for the project page alone: a
 * section a session writes appears beside the blueprint without a reload.
 */
export function useProjectSectionStream(onChange: (event: ProjectSectionStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('project-section', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ProjectSectionStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
