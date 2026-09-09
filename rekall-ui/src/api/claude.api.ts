import { apiClient, request } from './client'
import {
  ClaudeInstallationSchema,
  ClaudeMessageListSchema,
  ClaudeSessionListSchema,
  ClaudeSessionSchema,
  ClaudeUsageSchema
} from './schemas/claude.schema'
import type {
  ClaudeEffortChoice,
  ClaudeInstallation,
  ClaudeMessage,
  ClaudeModelChoice,
  ClaudeSession,
  ClaudeUsage
} from '@/model/claude'
import type { ClaudeSessionId, TaskId, TaskStepId } from '@/model/branded'

export async function fetchClaudeInstallation(): Promise<ClaudeInstallation> {
  return request(async () => ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude')))
}

export async function installClaudeIntegration(): Promise<ClaudeInstallation> {
  return request(async () =>
    ClaudeInstallationSchema.parse(await apiClient('/api/settings/claude/install', { method: 'POST' }))
  )
}

export interface StartClaudeSessionInput {
  stepId?: TaskStepId | null
  skipPermissions: boolean
  model?: ClaudeModelChoice
  effort?: ClaudeEffortChoice
}

export async function fetchClaudeUsage(): Promise<ClaudeUsage> {
  return request(async () => ClaudeUsageSchema.parse(await apiClient('/api/claude/usage')))
}

export async function fetchClaudeSessions(): Promise<ClaudeSession[]> {
  return request(async () => ClaudeSessionListSchema.parse(await apiClient('/api/claude/sessions')))
}

export async function fetchTaskClaudeSessions(taskId: TaskId): Promise<ClaudeSession[]> {
  return request(async () =>
    ClaudeSessionListSchema.parse(await apiClient(`/api/tasks/${taskId}/claude/sessions`))
  )
}

export async function startClaudeSession(
  taskId: TaskId,
  input: StartClaudeSessionInput
): Promise<ClaudeSession> {
  return request(async () =>
    ClaudeSessionSchema.parse(
      await apiClient(`/api/tasks/${taskId}/claude/sessions`, {
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

export async function fetchClaudeTranscript(sessionId: ClaudeSessionId): Promise<ClaudeMessage[]> {
  return request(async () =>
    ClaudeMessageListSchema.parse(await apiClient(`/api/claude/sessions/${sessionId}/messages`))
  )
}

export async function sendClaudePrompt(sessionId: ClaudeSessionId, text: string): Promise<void> {
  await request(() =>
    apiClient(`/api/claude/sessions/${sessionId}/prompt`, { method: 'POST', body: { text } })
  )
}

export async function clearClaudeSession(sessionId: ClaudeSessionId): Promise<ClaudeSession> {
  return request(async () =>
    ClaudeSessionSchema.parse(
      await apiClient(`/api/claude/sessions/${sessionId}/clear`, { method: 'POST' })
    )
  )
}

export async function stopClaudeSession(sessionId: ClaudeSessionId): Promise<ClaudeSession> {
  return request(async () =>
    ClaudeSessionSchema.parse(
      await apiClient(`/api/claude/sessions/${sessionId}/stop`, { method: 'POST' })
    )
  )
}

export async function deleteClaudeSession(sessionId: ClaudeSessionId): Promise<void> {
  await request(() => apiClient(`/api/claude/sessions/${sessionId}`, { method: 'DELETE' }))
}
