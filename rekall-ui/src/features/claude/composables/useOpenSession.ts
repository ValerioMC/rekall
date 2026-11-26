import { useRouter } from 'vue-router'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useChatPaneShown } from '@/features/chat/composables/useChatPaneShown'
import { useOpenSessionInPane } from '@/features/claude/composables/useOpenSessionInPane'
import { useTerminalPaneShown } from '@/features/claude/composables/useTerminalPaneShown'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * Where a session is shown: a chat full-size on the console, a task's terminal in the floating
 * panel above the dock (full-size when a session already is), and a terminal on a project alone on
 * the Diagrams screen, which is the only thing such a terminal does.
 */
export function useOpenSession(): (session: Terminal) => void {
  const terminals = useTerminalStore()
  const router = useRouter()
  const openInPane = useOpenSessionInPane()
  const terminalOnScreen = useTerminalPaneShown()
  const chatOnScreen = useChatPaneShown()
  return (session) => {
    if (session.kind === 'CHAT') {
      openInPane(session)
      return
    }
    if (!session.taskId) {
      void router.push({ name: 'diagrams', query: { session: session.id } })
      return
    }
    if (terminalOnScreen.value || chatOnScreen.value) openInPane(session)
    else terminals.float(session.id)
  }
}
