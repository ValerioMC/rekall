import { computed, type ComputedRef } from 'vue'
import { preferredProvider } from '@/common/config/session-launch'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskId } from '@/model/branded'

export function useRunProvider(taskId: () => TaskId | null): ComputedRef<SessionProvider> {
  const terminals = useTerminalStore()
  return computed(() => terminals.terminalForTask(taskId())?.provider ?? preferredProvider())
}
