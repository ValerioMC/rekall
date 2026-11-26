import { computed, type ComputedRef } from 'vue'
import { preferredProvider } from '@/common/config/session-launch'
import { useTaskProviderStore } from '@/features/claude/store/taskProvider.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskId } from '@/model/branded'

export interface RunProvider {
  provider: ComputedRef<SessionProvider>
  isHeld: ComputedRef<boolean>
}

/** What Run here starts on a task. A live session holds it: the task takes no other provider until that session closes. */
export function useRunProvider(taskId: () => TaskId | null): RunProvider {
  const terminals = useTerminalStore()
  const chosen = useTaskProviderStore()
  const session = computed(() => terminals.terminalForTask(taskId()))

  const provider = computed(() => {
    if (session.value) return session.value.provider
    const id = taskId()
    return id ? chosen.providerFor(id) : preferredProvider()
  })
  const isHeld = computed(() => session.value !== null)

  return { provider, isHeld }
}
