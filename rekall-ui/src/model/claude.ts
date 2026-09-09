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

export const CLAUDE_USAGE_STATUSES = ['OK', 'UNAUTHENTICATED', 'UNAVAILABLE'] as const
export type ClaudeUsageStatus = (typeof CLAUDE_USAGE_STATUSES)[number]

export const CLAUDE_USAGE_SEVERITIES = ['NORMAL', 'WARNING', 'CRITICAL'] as const
export type ClaudeUsageSeverity = (typeof CLAUDE_USAGE_SEVERITIES)[number]

export interface ClaudeUsageLimit {
  readonly key: string
  readonly label: string
  readonly percent: number
  readonly severity: ClaudeUsageSeverity
  readonly resetsAt: string | null
}

export interface ClaudeUsage {
  readonly status: ClaudeUsageStatus
  readonly limits: readonly ClaudeUsageLimit[]
  readonly fetchedAt: string
}

/** The session window is the one the meter shows at rest; the rest live in the popover. */
export function claudeSessionUsage(usage: ClaudeUsage): ClaudeUsageLimit | null {
  return usage.limits.find((limit) => limit.key === 'session') ?? usage.limits[0] ?? null
}

export const CLAUDE_SESSION_STATUSES = ['STARTING', 'WORKING', 'READY', 'EXITED', 'FAILED'] as const
export type ClaudeSessionStatus = (typeof CLAUDE_SESSION_STATUSES)[number]

/**
 * What a new in-app session can be pointed at. Each value but 'default' is a Claude Code
 * `--model` alias that resolves to the latest model of that family, so no version is pinned
 * here; 'default' leaves the signed-in account's own setting alone.
 */
export const CLAUDE_MODEL_CHOICES = ['default', 'sonnet', 'fable', 'opus', 'haiku'] as const
export type ClaudeModelChoice = (typeof CLAUDE_MODEL_CHOICES)[number]

const CLAUDE_MODEL_CHOICE_LABEL: Readonly<Record<ClaudeModelChoice, string>> = {
  default: 'Account default',
  sonnet: 'Sonnet',
  fable: 'Fable',
  opus: 'Opus',
  haiku: 'Haiku'
}

export function claudeModelChoiceLabel(choice: ClaudeModelChoice): string {
  return CLAUDE_MODEL_CHOICE_LABEL[choice]
}

/** The `--effort` levels Claude Code defines, plus 'default' for leaving it unset. */
export const CLAUDE_EFFORT_CHOICES = ['default', 'low', 'medium', 'high', 'xhigh', 'max'] as const
export type ClaudeEffortChoice = (typeof CLAUDE_EFFORT_CHOICES)[number]

const CLAUDE_EFFORT_CHOICE_LABEL: Readonly<Record<ClaudeEffortChoice, string>> = {
  default: 'Account default',
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  xhigh: 'Extra-high',
  max: 'Max'
}

export function claudeEffortChoiceLabel(choice: ClaudeEffortChoice): string {
  return CLAUDE_EFFORT_CHOICE_LABEL[choice]
}

/** A readable name for the effort level a session recorded, null when it ran at the default. */
export function claudeEffortLabel(effort: string | null | undefined): string | null {
  if (!effort) return null
  return (CLAUDE_EFFORT_CHOICE_LABEL as Record<string, string>)[effort] ?? effort
}

/**
 * A short, readable name for whatever the session recorded as its model: the alias it was
 * started with, or the concrete id Claude Code reports once it is running (`claude-sonnet-5`
 * reads as `Sonnet 5`, `claude-fable-5-1` as `Fable 5.1`). Unrecognised strings come back
 * as-is, null when there is nothing to show. No family or version is enumerated here: the id
 * is parsed, so a new model reads correctly the day it ships.
 */
export function claudeModelLabel(model: string | null | undefined): string | null {
  if (!model) return null
  const oneMillion = /\[1m\]/i.test(model) ? ' (1M)' : ''
  const stripped = model.replace(/-?\d{8}$/, '').replace(/\[1m\]/i, '')
  const match = /(opus|sonnet|haiku|fable)(?:[-@](\d+)(?:[-.](\d+))?)?/i.exec(stripped)
  const name = match?.[1]
  if (!name) return model
  const family = name.charAt(0).toUpperCase() + name.slice(1).toLowerCase()
  const major = match[2]
  const minor = match[3]
  const version = major ? ` ${major}${minor ? `.${minor}` : ''}` : ''
  return `${family}${version}${oneMillion}`
}

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
  readonly model: string | null
  readonly effort: string | null
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

export interface ClaudeToolMeta {
  /** The id of the call a TOOL_RESULT answers, or that a TOOL_USE waits on. */
  readonly toolUseId?: string
  /** Set on a TOOL_RESULT the CLI flagged as an error. */
  readonly error?: boolean
}

export function parseToolMeta(meta: string | null): ClaudeToolMeta {
  if (!meta) return {}
  try {
    const parsed = JSON.parse(meta) as unknown
    return parsed && typeof parsed === 'object' ? (parsed as ClaudeToolMeta) : {}
  } catch {
    return {}
  }
}

/** The one field of a tool's input worth reading at a glance: the command, the path, the query. */
const TOOL_DETAIL_FIELDS: Readonly<Record<string, readonly string[]>> = {
  Bash: ['command'],
  Read: ['file_path'],
  Edit: ['file_path'],
  MultiEdit: ['file_path'],
  Write: ['file_path'],
  NotebookEdit: ['notebook_path'],
  Glob: ['pattern'],
  Grep: ['pattern'],
  Task: ['description'],
  Skill: ['command', 'skill'],
  WebFetch: ['url'],
  WebSearch: ['query']
}

/**
 * A one-line reading of a tool call's input, so the collapsed transcript row says what the call
 * did without being opened. Returns null when the input is not JSON or holds nothing worth showing.
 */
export function claudeToolDetail(toolName: string | null, content: string | null): string | null {
  if (!content) return null
  let input: unknown
  try {
    input = JSON.parse(content)
  } catch {
    return null
  }
  if (!input || typeof input !== 'object') return null
  const record = input as Record<string, unknown>
  const preferred = TOOL_DETAIL_FIELDS[toolName ?? ''] ?? []
  const keys = [...preferred, ...Object.keys(record).filter((key) => !preferred.includes(key))]
  for (const key of keys) {
    const value = record[key]
    if (typeof value === 'string' && value.trim()) {
      return value.trim().replace(/\s+/g, ' ')
    }
  }
  return null
}

export interface ClaudeTurnStats {
  readonly subtype?: string
  readonly isError?: boolean
  readonly durationMs?: number
  readonly numTurns?: number
  readonly costUsd?: number
  readonly totalTokens?: number
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
