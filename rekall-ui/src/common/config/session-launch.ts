import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { SESSION_PROVIDERS, type SessionProvider } from '@/features/claude/model/session-provider'
import type { SessionKind } from '@/features/claude/model/terminal'

const PROVIDER_KEY = 'rekall.session.provider'
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

export function preferredProvider(): SessionProvider {
  const stored = readChoice(PROVIDER_KEY)
  return SESSION_PROVIDERS.find((provider) => provider === stored) ?? 'CLAUDE'
}

export function setPreferredProvider(value: SessionProvider): void {
  saveChoice(PROVIDER_KEY, value)
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
    ? { ...codexPreferences(), provider, kind: 'TERMINAL' }
    : {
        provider,
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort(),
        kind: preferredLaunchKind()
      }
}
