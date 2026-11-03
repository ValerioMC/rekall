import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useChatWindowShown } from '@/features/chat/composables/useChatWindowShown'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'
import { CHAT_ID, chatSession } from './support'
import { installStorage } from '../../support/memoryStorage'

const OWN = 't-1' as TaskId
const OTHER = 't-2' as TaskId

describe('the chat window follows its task', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
  })

  it('shows on the task the chat session belongs to and disappears on another', () => {
    useTerminalStore().upsert(chatSession())
    useChatStore().select(CHAT_ID)
    const console = useConsoleStore()
    const shown = useChatWindowShown()

    console.selectedTaskId = OWN
    expect(shown.value).toBe(true)
    console.selectedTaskId = OTHER
    expect(shown.value).toBe(false)
    console.selectedTaskId = OWN
    expect(shown.value).toBe(true)
  })

  it('shows the task\'s own live chat even when another one was the active one', () => {
    useTerminalStore().upsert(chatSession())
    useTerminalStore().upsert(chatSession({ id: 'chat-2' as never, taskId: OTHER }))
    useChatStore().select(CHAT_ID)
    useConsoleStore().selectedTaskId = OTHER

    expect(useChatWindowShown().value).toBe(true)
    useChatStore().showTaskSession(OTHER)
    expect(useChatStore().activeChatId).toBe('chat-2')
  })

  it('stays on an ended session of the task until it is closed', () => {
    useTerminalStore().upsert(chatSession({ live: false }))
    useChatStore().select(CHAT_ID)
    useConsoleStore().selectedTaskId = OWN

    expect(useChatWindowShown().value).toBe(true)
  })

  it('opens empty only on the task that asked for it', () => {
    const console = useConsoleStore()
    const shown = useChatWindowShown()
    console.selectedTaskId = OWN

    useChatDockStore().open(OWN)
    expect(shown.value).toBe(true)
    console.selectedTaskId = OTHER
    expect(shown.value).toBe(false)
  })

  it('stays hidden when hidden, whatever the task', () => {
    useTerminalStore().upsert(chatSession())
    useConsoleStore().selectedTaskId = OWN
    useChatDockStore().hide()

    expect(useChatWindowShown().value).toBe(false)
  })

  it('keeps a session that belongs to no task on screen, and gives way to a task\'s own', () => {
    useTerminalStore().upsert(chatSession({ taskId: null }))
    useChatStore().select(CHAT_ID)
    const console = useConsoleStore()
    const shown = useChatWindowShown()

    console.selectedTaskId = OWN
    expect(shown.value).toBe(true)
    useTerminalStore().upsert(chatSession({ id: 'chat-2' as never }))
    useChatStore().showTaskSession(OWN)
    expect(useChatStore().activeChatId).toBe('chat-2')
  })

  it('lets go of another task\'s session when the task has none', () => {
    useTerminalStore().upsert(chatSession())
    useChatStore().select(CHAT_ID)

    useChatStore().showTaskSession(OTHER)

    expect(useChatStore().activeChatId).toBeNull()
  })
})
