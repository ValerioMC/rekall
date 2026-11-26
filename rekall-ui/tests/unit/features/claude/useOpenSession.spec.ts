import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { effectScope } from 'vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId, TerminalId } from '@/model/branded'
import { chatSession } from '../chat/support'
import { taskFixture } from '../../support/taskFixture'

const route = { name: 'projects' }
const push = vi.fn()

vi.mock('vue-router', () => ({
  useRouter: () => ({ currentRoute: { value: route }, push })
}))

const TASK = 't-1' as TaskId
const TERMINAL = 'term-1' as TerminalId
const LOOSE = 'chat-9' as TerminalId

const terminal = chatSession({ id: TERMINAL, kind: 'TERMINAL' })

function open() {
  return effectScope().run(() => useOpenSession())!
}

describe('opening a session', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().tasks = [taskFixture(TASK)]
    route.name = 'projects'
    push.mockReset()
  })

  it('takes a task\'s chat to the console, full-size', () => {
    open()(chatSession())

    expect([useConsoleStore().selectedTaskId, useConsoleStore().paneFocus]).toEqual([TASK, 'chat'])
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('takes a chat on no task there too, named by its session and with no task in view', () => {
    useConsoleStore().selectTask(TASK)

    open()(chatSession({ id: LOOSE, taskId: null }))

    expect(useConsoleStore().location).toMatchObject({ pane: 'chat', taskId: null, chatId: LOOSE })
    expect(useChatStore().activeChatId).toBe(LOOSE)
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('minimizes a floating terminal when a chat takes the screen', () => {
    useTerminalStore().float(TERMINAL)

    open()(chatSession())

    expect(useTerminalStore().floatingId).toBeNull()
  })

  it('floats a task\'s terminal over the page', () => {
    open()(terminal)

    expect(useTerminalStore().floatingId).toBe(TERMINAL)
    expect(push).not.toHaveBeenCalled()
  })

  it('shows a terminal full-size when a chat already is', () => {
    route.name = 'console'
    useConsoleStore().selectTask(TASK)
    useConsoleStore().openChat()

    open()(terminal)

    expect(useConsoleStore().paneFocus).toBe('terminal')
    expect(useTerminalStore().activeTerminalId).toBe(TERMINAL)
    expect(useTerminalStore().floatingId).toBeNull()
  })

  it('sends a terminal on no task to the Diagrams screen', () => {
    open()(chatSession({ id: TERMINAL, kind: 'TERMINAL', taskId: null }))

    expect(push).toHaveBeenCalledWith({ name: 'diagrams', query: { session: TERMINAL } })
  })
})
