import {
  CLAUDE_EFFORT_CHOICES,
  SESSION_VIEWS,
  type ClaudeEffortChoice,
  type SessionView
} from '@/features/claude/model/claude'
import type { SessionKind } from '@/features/claude/model/terminal'

const STORAGE_KEY = 'rekall.claude.skip-permissions'
const MODEL_KEY = 'rekall.claude.model'
const EFFORT_KEY = 'rekall.claude.effort'
const SESSION_KIND_KEY = 'rekall.claude.session-kind'
// The shape a launch accepts (`claude_model_choice` on the server), so any release's value is kept.
const MODEL_VALUE = /^[a-z0-9][a-z0-9_.:/@-]*(\[1m\])?$/i
const MODEL_MAX_LENGTH = 150

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
export function preferredModel(): string {
  try {
    const stored = window.localStorage.getItem(MODEL_KEY) ?? ''
    return stored.length <= MODEL_MAX_LENGTH && MODEL_VALUE.test(stored) ? stored : 'default'
  } catch {
    return 'default'
  }
}

export function setPreferredModel(value: string): void {
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

/** The view for a new in-app session. */
export function preferredSessionKind(): SessionView {
  try {
    const stored = window.localStorage.getItem(SESSION_KIND_KEY)
    return (SESSION_VIEWS as readonly string[]).includes(stored ?? '')
      ? (stored as SessionView)
      : 'terminal'
  } catch {
    return 'terminal'
  }
}

export function setPreferredSessionKind(value: SessionView): void {
  try {
    window.localStorage.setItem(SESSION_KIND_KEY, value)
  } catch {
  }
}

/** The preference as the `kind` an open request carries. */
export function preferredLaunchKind(): SessionKind {
  return preferredSessionKind() === 'chat' ? 'CHAT' : 'TERMINAL'
}
