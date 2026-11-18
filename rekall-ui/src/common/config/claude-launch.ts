import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  CLAUDE_SESSION_VIEWS,
  type ClaudeEffortChoice,
  type ClaudeModelChoice,
  type ClaudeSessionView
} from '@/features/claude/model/claude'
import type { SessionKind } from '@/features/claude/model/terminal'

const STORAGE_KEY = 'rekall.claude.skip-permissions'
const MODEL_KEY = 'rekall.claude.model'
const EFFORT_KEY = 'rekall.claude.effort'
const SESSION_KIND_KEY = 'rekall.claude.session-kind'

export function skipsPermissions(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === 'true'
  } catch {
    return false
  }
}

export function setSkipsPermissions(value: boolean): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, String(value))
  } catch {
  }
}

/** The model a new in-app session is started with. 'default' leaves the account setting alone. */
export function preferredModel(): ClaudeModelChoice {
  try {
    const stored = window.localStorage.getItem(MODEL_KEY)
    return (CLAUDE_MODEL_CHOICES as readonly string[]).includes(stored ?? '')
      ? (stored as ClaudeModelChoice)
      : 'default'
  } catch {
    return 'default'
  }
}

export function setPreferredModel(value: ClaudeModelChoice): void {
  try {
    window.localStorage.setItem(MODEL_KEY, value)
  } catch {
  }
}

/** The `--effort` level a new in-app session is started with. 'default' leaves it unset. */
export function preferredEffort(): ClaudeEffortChoice {
  try {
    const stored = window.localStorage.getItem(EFFORT_KEY)
    return (CLAUDE_EFFORT_CHOICES as readonly string[]).includes(stored ?? '')
      ? (stored as ClaudeEffortChoice)
      : 'default'
  } catch {
    return 'default'
  }
}

export function setPreferredEffort(value: ClaudeEffortChoice): void {
  try {
    window.localStorage.setItem(EFFORT_KEY, value)
  } catch {
  }
}

/** How a new in-app session runs `claude`: in a terminal, the default, or headless in the chat window. */
export function preferredSessionKind(): ClaudeSessionView {
  try {
    const stored = window.localStorage.getItem(SESSION_KIND_KEY)
    return (CLAUDE_SESSION_VIEWS as readonly string[]).includes(stored ?? '')
      ? (stored as ClaudeSessionView)
      : 'terminal'
  } catch {
    return 'terminal'
  }
}

export function setPreferredSessionKind(value: ClaudeSessionView): void {
  try {
    window.localStorage.setItem(SESSION_KIND_KEY, value)
  } catch {
  }
}

/** The preference as the `kind` an open request carries. */
export function preferredLaunchKind(): SessionKind {
  return preferredSessionKind() === 'chat' ? 'CHAT' : 'TERMINAL'
}
