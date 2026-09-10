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

/** Claude Code `--model` aliases plus 'default' for leaving the account setting alone. */
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

/** A readable name for the effort level, null when there is nothing to show. */
export function claudeEffortLabel(effort: string | null | undefined): string | null {
  if (!effort) return null
  return (CLAUDE_EFFORT_CHOICE_LABEL as Record<string, string>)[effort] ?? effort
}

/**
 * A short, readable name for a Claude Code model id (`claude-sonnet-5` -> `Sonnet 5`). The id is
 * parsed, not enumerated; unrecognised strings come back as-is, null when there is nothing to show.
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
