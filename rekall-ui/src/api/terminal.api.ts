import { apiClient, request } from './client'
import { TerminalListSchema, TerminalSchema } from './schemas/terminal.schema'
import { env } from '@/common/config/env'
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

function terminalSocketUrl(id: TerminalId): string {
  const base = env.VITE_API_BASE_URL
  if (base) return `${base.replace(/^http/, 'ws')}/api/terminal/${id}/io`
  const scheme = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${scheme}//${window.location.host}/api/terminal/${id}/io`
}

/**
 * Type a line into a live terminal's stdin without holding a pane open on it: a short-lived second
 * attachment to the same PTY, sent and torn down. `PtyTerminalManager` fans output to every
 * attached socket, so this never disturbs whatever pane already has the terminal open.
 */
export function sendTerminalInput(id: TerminalId, text: string): Promise<void> {
  return new Promise((resolve, reject) => {
    let opened = false
    const socket = new WebSocket(terminalSocketUrl(id))
    const timeout = window.setTimeout(() => {
      if (opened) return
      socket.close()
      reject(new Error('This terminal did not respond in time.'))
    }, 5000)

    socket.onopen = () => {
      opened = true
      window.clearTimeout(timeout)
      socket.send(new TextEncoder().encode(text))
      window.setTimeout(() => socket.close(), 150)
      resolve()
    }
    socket.onerror = () => {
      if (opened) return
      window.clearTimeout(timeout)
      reject(new Error('Could not reach this terminal.'))
    }
  })
}
