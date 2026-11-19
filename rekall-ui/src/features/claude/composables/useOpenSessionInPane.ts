import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'

/** Shows a terminal session of a task in the console's full terminal pane. */
export function useOpenSessionInPane(): (terminal: Terminal) => void {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const router = useRouter()
  return (terminal) => {
    if (!terminal.taskId) return
    store.selectTask(terminal.taskId)
    store.openTerminal()
    terminals.select(terminal.id)
    terminals.unfloat()
    if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
  }
}
