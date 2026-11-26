import { computed } from 'vue'
import { preferredProvider } from '@/common/config/session-launch'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, sessionViewLabel, type SessionProvider } from '@/features/claude/model/session-provider'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useCodexStore } from '@/features/codex/store/codex.store'
import type { TaskId } from '@/model/branded'

export function useSessionProviderOptions(taskId: () => TaskId | null, queueing: () => boolean = () => false) {
  const claude = useClaudeConnectionStore()
  const codex = useCodexStore()
  const terminals = useTerminalStore()
  const live = computed(() => terminals.terminalForTask(taskId()))

  function missing(provider: SessionProvider): string | null {
    const status = provider === 'CLAUDE' ? claude.installation?.status : codex.setup?.status
    return status === 'CLI_MISSING' ? `${SESSION_PROVIDER_LABEL[provider]} CLI is not installed.` : null
  }

  function describe(provider: SessionProvider): string | null {
    const session = live.value
    if (!session) return null
    const name = `${SESSION_PROVIDER_LABEL[session.provider]} ${sessionViewLabel(session.kind)}`
    if (queueing()) return `${name} is open on this task. Close it before the queue starts this task.`
    if (session.provider !== provider) return `${name} is open on this task. Close it to use ${SESSION_PROVIDER_LABEL[provider]}.`
    return `${name} is open on this task.`
  }

  function options() {
    const preferred = preferredProvider()
    return SESSION_PROVIDERS.map((provider) => ({
      provider,
      preferred: provider === preferred,
      unavailable: missing(provider),
      detail: describe(provider)
    }))
  }

  return { options, missing }
}
