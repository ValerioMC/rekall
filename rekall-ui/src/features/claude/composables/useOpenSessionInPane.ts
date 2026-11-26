import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

/** Shows a session full-size on the console: a task's terminal or chat, or a chat on a project alone. */
export function useOpenSessionInPane(): (session: Terminal) => void {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const chat = useChatStore()
  const router = useRouter()

  function showChat(session: Terminal): void {
    if (session.taskId) {
      store.selectTask(session.taskId)
      store.openChat()
    } else store.openProjectChat(session.id)
    chat.select(session.id)
  }

  function showTerminal(session: Terminal, taskId: TaskId): void {
    store.selectTask(taskId)
    store.openTerminal()
    terminals.select(session.id)
  }

  return (session) => {
    if (session.kind === 'CHAT') showChat(session)
    else if (session.taskId) showTerminal(session, session.taskId)
    else return
    terminals.unfloat()
    if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
  }
}
