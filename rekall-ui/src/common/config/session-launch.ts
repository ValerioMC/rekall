import { shallowRef } from 'vue'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { SESSION_PROVIDERS, type SessionProvider } from '@/features/claude/model/session-provider'
import type { SessionKind } from '@/features/claude/model/terminal'

const PROVIDER_KEY = 'rekall.session.provider'
const providerPreference = shallowRef<{ provider: SessionProvider; saved: boolean }>()
let lastStoredProvider: SessionProvider = 'CLAUDE'
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

function readChoice(key: string): string | null | undefined {
  try {
    return window.localStorage.getItem(key)
  } catch {
    return undefined
  }
}

function saveChoice(key: string, value: string): boolean {
  try {
    window.localStorage.setItem(key, value)
    return true
  } catch {
    return false
  }
}

export function preferredProvider(): SessionProvider {
  const preference = providerPreference.value
  if (preference && !preference.saved) return preference.provider
  const stored = readChoice(PROVIDER_KEY)
  if (stored === undefined) return lastStoredProvider
  lastStoredProvider = SESSION_PROVIDERS.find((provider) => provider === stored) ?? 'CLAUDE'
  return lastStoredProvider
}

export function setPreferredProvider(value: SessionProvider): void {
  const saved = saveChoice(PROVIDER_KEY, value)
  lastStoredProvider = value
  providerPreference.value = { provider: value, saved }
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
