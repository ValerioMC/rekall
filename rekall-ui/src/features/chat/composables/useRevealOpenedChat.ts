import { watch } from 'vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'

/** Brings every chat session Rekall opens, or reuses for a plan, to the window as it happens. */
export function useRevealOpenedChat(): void {
  const terminals = useTerminalStore()
  const chat = useChatStore()
  watch(
    () => terminals.lastOpened,
    (opened) => {
      if (opened?.kind === 'CHAT') chat.reveal(opened)
    }
  )
}
