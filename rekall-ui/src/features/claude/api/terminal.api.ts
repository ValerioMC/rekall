import { apiClient, request } from '../../../api/client'
import { TerminalListSchema, TerminalSchema } from './schemas/terminal.schema'
import { env } from '@/common/config/env'
import type { Terminal, TerminalMode, TerminalTarget } from '@/features/claude/model/terminal'
import type { DiagramKind } from '@/features/diagram/model/diagram'
import type { ClaudeEffortChoice, ClaudeModelChoice } from '@/features/claude/model/claude'
import type { TaskStepId, TerminalId } from '@/model/branded'

export interface OpenTerminalInput {
  stepId?: TaskStepId | null
  skipPermissions: boolean
  model?: ClaudeModelChoice
  effort?: ClaudeEffortChoice
  mode?: TerminalMode
  /** What a `GENERATE` terminal asks the session to draw, and as which diagram types. */
  request?: string
  kinds?: readonly DiagramKind[]
  /** The GHSA id an `ASSESS` terminal asks the session to judge. */
  advisory?: string
}

function terminalsPath(target: TerminalTarget): string {
  return 'taskId' in target ? `/api/tasks/${target.taskId}/terminals` : `/api/projects/${target.projectId}/terminals`
}

export async function fetchTerminals(): Promise<Terminal[]> {
  return request(async () => TerminalListSchema.parse(await apiClient('/api/terminals')))
}

export async function openTerminal(target: TerminalTarget, input: OpenTerminalInput): Promise<Terminal> {
  return request(async () =>
    TerminalSchema.parse(
      await apiClient(terminalsPath(target), {
        method: 'POST',
        body: {
          stepId: input.stepId ?? null,
          skipPermissions: input.skipPermissions,
          model: input.model && input.model !== 'default' ? input.model : null,
          effort: input.effort && input.effort !== 'default' ? input.effort : null,
          mode: input.mode ?? 'WORK',
          request: input.request ?? null,
          kinds: input.kinds ?? null,
          advisory: input.advisory ?? null
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
