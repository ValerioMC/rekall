import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useCodexStore } from '@/features/codex/store/codex.store'

export function useSessionProviderOptions() {
  const claude = useClaudeConnectionStore()
  const codex = useCodexStore()

  function missing(provider: SessionProvider): string | null {
    const status = provider === 'CLAUDE' ? claude.installation?.status : codex.setup?.status
    return status === 'CLI_MISSING' ? `${SESSION_PROVIDER_LABEL[provider]} CLI is not installed.` : null
  }

  function options() {
    return SESSION_PROVIDERS.map((provider) => ({
      provider,
      unavailable: missing(provider)
    }))
  }

  return { options, missing }
}
