import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { SESSION_PROVIDERS, type SessionProvider } from '@/features/claude/model/session-provider'
import type { SessionKind } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

const PROVIDER_KEY = 'rekall.session.provider'
const TASK_PROVIDER_KEY = 'rekall.session.task-provider.'
const CODEX_MODEL_KEY = 'rekall.codex.model'
const CODEX_EFFORT_KEY = 'rekall.codex.effort'
const CODEX_BYPASS_KEY = 'rekall.codex.bypass'

export interface LaunchChoices {
  readonly skipPermissions: boolean
  readonly provider: SessionProvider
  readonly model: string
  readonly effort: string
  readonly kind: SessionKind
}

interface CodexPreferences {
  readonly model: string
  readonly effort: string
  readonly skipPermissions: boolean
}

function readChoice(key: string): string | null {
  try {
    return window.localStorage.getItem(key)
  } catch {
    return null
  }
}

function saveChoice(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value)
  } catch {
  }
}

function readProvider(key: string): SessionProvider | null {
  const stored = readChoice(key)
  return SESSION_PROVIDERS.find((provider) => provider === stored) ?? null
}

export function preferredProvider(): SessionProvider {
  return readProvider(PROVIDER_KEY) ?? 'CLAUDE'
}

export function setPreferredProvider(value: SessionProvider): void {
  saveChoice(PROVIDER_KEY, value)
}

export function savedTaskProvider(taskId: TaskId): SessionProvider | null {
  return readProvider(TASK_PROVIDER_KEY + taskId)
}

export function saveTaskProvider(taskId: TaskId, value: SessionProvider): void {
  saveChoice(TASK_PROVIDER_KEY + taskId, value)
}

export function codexPreferences(): CodexPreferences {
  return {
    model: readChoice(CODEX_MODEL_KEY) ?? 'default',
    effort: readChoice(CODEX_EFFORT_KEY) ?? 'default',
    skipPermissions: readChoice(CODEX_BYPASS_KEY) === 'true'
  }
}

export function setCodexModel(model: string): void {
  saveChoice(CODEX_MODEL_KEY, model)
}

export function setCodexEffort(effort: string): void {
  saveChoice(CODEX_EFFORT_KEY, effort)
}

export function setCodexBypass(enabled: boolean): void {
  saveChoice(CODEX_BYPASS_KEY, String(enabled))
}

export function launchChoices(provider = preferredProvider()): LaunchChoices {
  return provider === 'CODEX'
    ? { ...codexPreferences(), provider, kind: preferredLaunchKind() }
    : {
        provider,
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort(),
        kind: preferredLaunchKind()
      }
}
