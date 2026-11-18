import { computed, type ComputedRef } from 'vue'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useCodexStore } from '@/features/codex/store/codex.store'

export function useSessionProviderChoice(): ComputedRef<boolean> {
  const claude = useClaudeConnectionStore()
  const codex = useCodexStore()

  return computed(() =>
    claude.installation !== null && claude.installation.status !== 'CLI_MISSING' &&
    codex.setup !== null && codex.setup.status !== 'CLI_MISSING'
  )
}
