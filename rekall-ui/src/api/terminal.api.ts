import { apiClient, request } from './client'
import { TerminalListSchema, TerminalSchema } from './schemas/terminal.schema'
import type { Terminal } from '@/model/terminal'
import type { ClaudeEffortChoice, ClaudeModelChoice } from '@/model/claude'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

export interface OpenTerminalInput {
  stepId?: TaskStepId | null
  skipPermissions: boolean
  model?: ClaudeModelChoice
  effort?: ClaudeEffortChoice
}

export async function fetchTerminals(): Promise<Terminal[]> {
  return request(async () => TerminalListSchema.parse(await apiClient('/api/terminals')))
}

export async function openTerminal(taskId: TaskId, input: OpenTerminalInput): Promise<Terminal> {
  return request(async () =>
    TerminalSchema.parse(
      await apiClient(`/api/tasks/${taskId}/terminals`, {
        method: 'POST',
        body: {
          stepId: input.stepId ?? null,
          skipPermissions: input.skipPermissions,
          model: input.model && input.model !== 'default' ? input.model : null,
          effort: input.effort && input.effort !== 'default' ? input.effort : null
        }
      })
    )
  )
}

export async function closeTerminal(id: TerminalId): Promise<void> {
  await request(() => apiClient(`/api/terminals/${id}`, { method: 'DELETE' }))
}
