import { z } from 'zod'
import { apiClient, request } from '../../../api/client'
import { TaskSchema } from './schemas/catalog.schema'
import { TerminalSchema } from '@/features/claude/api/schemas/terminal.schema'
import type { ClaudeEffortChoice, ClaudeModelChoice } from '@/features/claude/model/claude'
import type { SessionKind, Terminal } from '@/features/claude/model/terminal'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

export interface QuickTaskInput {
  text: string
  skipPermissions: boolean
  model?: ClaudeModelChoice
  effort?: ClaudeEffortChoice
  kind?: SessionKind
}

/** The task a quick task made, and the session already opened on it. */
export interface QuickTaskStarted {
  readonly task: Task
  readonly terminal: Terminal
}

const QuickTaskStartedSchema = z.object({ task: TaskSchema, terminal: TerminalSchema })

export async function createQuickTask(projectId: ProjectId, input: QuickTaskInput): Promise<QuickTaskStarted> {
  return request(async () =>
    QuickTaskStartedSchema.parse(
      await apiClient(`/api/projects/${projectId}/quick-tasks`, {
        method: 'POST',
        body: {
          text: input.text,
          skipPermissions: input.skipPermissions,
          model: input.model && input.model !== 'default' ? input.model : null,
          effort: input.effort && input.effort !== 'default' ? input.effort : null,
          kind: input.kind ?? 'TERMINAL'
        }
      })
    )
  )
}
