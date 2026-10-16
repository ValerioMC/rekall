import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * Where a session is shown: one on a task in the console's terminal pane, one on a project alone
 * on the Diagrams screen, which is the only thing such a session does.
 */
export function useOpenSession(): (terminal: Terminal) => void {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const router = useRouter()
  return (terminal) => {
    if (!terminal.taskId) {
      void router.push({ name: 'diagrams', query: { session: terminal.id } })
      return
    }
    store.selectTask(terminal.taskId)
    store.openTerminal()
    terminals.select(terminal.id)
    if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
  }
}
