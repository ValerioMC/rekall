import type { ClaudeMessageId, ClaudeSessionId, TaskId, TaskStepId } from '@/model/branded'

export type ClaudeConnectionStatus = 'CONNECTED' | 'OUTDATED' | 'NOT_CONNECTED' | 'CLI_MISSING'

export interface ClaudeInstallation {
  readonly status: ClaudeConnectionStatus
  readonly endpoint: string
  readonly registeredUrl: string | null
  readonly folderScoped: readonly string[]
  readonly commandInstalled: boolean
  readonly cliPath: string | null
  readonly manualCommand: string
}

export const CLAUDE_SESSION_STATUSES = ['STARTING', 'WORKING', 'READY', 'EXITED', 'FAILED'] as const
export type ClaudeSessionStatus = (typeof CLAUDE_SESSION_STATUSES)[number]

export const CLAUDE_MESSAGE_ROLES = [
  'USER',
  'ASSISTANT',
  'TOOL_USE',
  'TOOL_RESULT',
  'RESULT',
  'SYSTEM',
  'ERROR'
] as const
export type ClaudeMessageRole = (typeof CLAUDE_MESSAGE_ROLES)[number]

export interface ClaudeSession {
  readonly id: ClaudeSessionId
  readonly taskId: TaskId
  readonly taskLabel: string
  readonly taskTitle: string
  readonly projectLabel: string
  readonly anchor: string
  readonly stepId: TaskStepId | null
  readonly anchors: string
  readonly workingDir: string
  readonly cliSessionId: string | null
  readonly status: ClaudeSessionStatus
  readonly live: boolean
  readonly skipPermissions: boolean
  readonly detail: string | null
  readonly exitCode: number | null
  readonly lastActivityAt: string
  readonly startedAt: string
  readonly endedAt: string | null
  readonly updatedAt: string
}

export interface ClaudeMessage {
  readonly id: ClaudeMessageId
  readonly sessionId: ClaudeSessionId
  readonly seq: number
  readonly role: ClaudeMessageRole
  readonly content: string | null
  readonly toolName: string | null
  readonly meta: string | null
  readonly createdAt: string
}

export interface ClaudeTurnStats {
  readonly subtype?: string
  readonly isError?: boolean
  readonly durationMs?: number
  readonly numTurns?: number
  readonly costUsd?: number
}

export function claudeSessionIsLive(status: ClaudeSessionStatus): boolean {
  return status === 'STARTING' || status === 'WORKING' || status === 'READY'
}

export function claudeSessionAcceptsPrompt(status: ClaudeSessionStatus): boolean {
  return status === 'READY'
}

const CLAUDE_STATUS_LABEL: Readonly<Record<ClaudeSessionStatus, string>> = {
  STARTING: 'starting',
  WORKING: 'working',
  READY: 'ready',
  EXITED: 'ended',
  FAILED: 'failed'
}

export function claudeStatusLabel(status: ClaudeSessionStatus): string {
  return CLAUDE_STATUS_LABEL[status]
}

export function parseTurnStats(meta: string | null): ClaudeTurnStats | null {
  if (!meta) return null
  try {
    return JSON.parse(meta) as ClaudeTurnStats
  } catch {
    return null
  }
}
