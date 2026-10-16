import { onScopeDispose } from 'vue'
import { env } from '@/common/config/env'
import { ProjectSectionStreamEventSchema } from '@/features/project/api/schemas/project-section.schema'
import type { ProjectSectionStreamEvent } from '@/features/project/model/project-section'

/**
 * The `project-section` frames of the console's event stream, for the project page alone: a
 * section a session writes appears beside the blueprint without a reload.
 */
export function useProjectSectionStream(onChange: (event: ProjectSectionStreamEvent) => void): void {
  if (typeof EventSource === 'undefined') return
  const source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
  source.addEventListener('project-section', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ProjectSectionStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(() => source.close())
}
