import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { ref } from 'vue'
import ChatWindow from '@/features/chat/components/ChatWindow.vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'
import { CHAT_ID, MarkdownStub, chatSession } from './support'
import { taskFixture } from '../../support/taskFixture'
import { installStorage } from '../../support/memoryStorage'

const chatApi = { sendMessage: vi.fn(), interrupt: vi.fn(), answerPermission: vi.fn() }
const terminalApi = { fetchTerminals: vi.fn(), openTerminal: vi.fn(), closeTerminal: vi.fn() }
const consoleFake = vi.hoisted(() => ({ selectedTask: null as unknown, selectTask: null as unknown }))

vi.mock('@/features/chat/api/chat.api', () => ({
  sendMessage: (...a: unknown[]) => chatApi.sendMessage(...a),
  interrupt: (...a: unknown[]) => chatApi.interrupt(...a),
  answerPermission: (...a: unknown[]) => chatApi.answerPermission(...a)
}))

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => terminalApi.fetchTerminals(...a),
  openTerminal: (...a: unknown[]) => terminalApi.openTerminal(...a),
  closeTerminal: (...a: unknown[]) => terminalApi.closeTerminal(...a),
  sendTerminalInput: vi.fn()
}))

vi.mock('@/features/chat/composables/useChatEvents', () => ({
  useChatEvents: () => ({ connected: ref(true), reconnecting: ref(false) })
}))

vi.mock('@/stores/console.store', async () => {
  const { defineStore } = await import('pinia')
  const { ref: vueRef } = await import('vue')
  return {
    useConsoleStore: defineStore('console', () => {
      const selectedTask = vueRef(consoleFake.selectedTask)
      const selectedTaskSteps = vueRef([])
      const selectTask = (...a: unknown[]) => (consoleFake.selectTask as (...b: unknown[]) => void)(...a)
      return { selectedTask, selectedTaskSteps, selectTask }
    })
  }
})

const TASK = 't-1' as TaskId
const BASH = { tool: 'bash', command: 'ls', description: null } as const

function task(over: Partial<Task> = {}): Task {
  return { ...taskFixture(TASK), projectRepoFolder: '/code/vega', ...over }
}

function mountWindow(): VueWrapper {
  return mount(ChatWindow, {
    attachTo: document.body,
    global: { stubs: { AppMarkdownEditor: MarkdownStub, TerminalConnectionChips: true, RouterLink: true } }
  })
}

function openChat(): void {
  const terminals = useTerminalStore()
  terminals.upsert(chatSession())
  useChatStore().select(CHAT_ID)
}

function ingest(event: ChatEvent): void {
  useChatStore().ingest(CHAT_ID, event)
}

describe('ChatWindow', () => {
  let storage: Map<string, string>

  beforeEach(() => {
    setActivePinia(createPinia())
    storage = installStorage()
    document.body.innerHTML = ''
    Object.values(chatApi).forEach((mock) => mock.mockReset().mockResolvedValue(undefined))
    Object.values(terminalApi).forEach((mock) => mock.mockReset())
    consoleFake.selectedTask = task()
    consoleFake.selectTask = vi.fn()
  })

  it('sends the typed message through the store and clears the field', async () => {
    openChat()
    const chat = mountWindow()
    await flushPromises()

    await chat.get('[data-testid=chat-input]').setValue('hello')
    await chat.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(chatApi.sendMessage).toHaveBeenCalledWith(CHAT_ID, 'hello')
    expect((chat.get('[data-testid=chat-input]').element as HTMLTextAreaElement).value).toBe('')
    chat.unmount()
  })

  it('keeps the text and says why when the send fails', async () => {
    chatApi.sendMessage.mockRejectedValue(new Error('Session is gone'))
    openChat()
    const chat = mountWindow()

    await chat.get('[data-testid=chat-input]').setValue('hello')
    await chat.get('[data-testid=chat-send]').trigger('click')
    await flushPromises()

    expect((chat.get('[data-testid=chat-input]').element as HTMLTextAreaElement).value).toBe('hello')
    expect(chat.get('[data-testid=chat-send-error]').text()).toBe('Session is gone')
    chat.unmount()
  })

  it('stops the running turn through interrupt and shows Stopping until the turn ends', async () => {
    openChat()
    ingest({ seq: 1, type: 'userMessage', text: 'go' })
    const chat = mountWindow()
    await flushPromises()

    await chat.get('[data-testid=chat-stop]').trigger('click')
    await flushPromises()
    expect(chatApi.interrupt).toHaveBeenCalledWith(CHAT_ID)
    expect(chat.get('[data-testid=chat-stop]').text()).toBe('Stopping…')

    ingest({
      seq: 2,
      type: 'turnFinished',
      usage: { inputTokens: 1, cacheCreationInputTokens: 0, cacheReadInputTokens: 0, outputTokens: 1 },
      durationMs: 10,
      outcome: 'interrupted'
    })
    await flushPromises()
    expect(chat.find('[data-testid=chat-stop]').exists()).toBe(false)
    chat.unmount()
  })

  it('posts Allow, keeps the card disabled until the resolution arrives, then clears it', async () => {
    openChat()
    ingest({ seq: 1, type: 'permissionRequested', requestId: 'req-1', toolUseId: 'u1', name: 'Bash', input: BASH })
    const chat = mountWindow()
    await flushPromises()

    expect(chat.get('[data-testid=chat-status]').text()).toBe('waiting for approval')
    expect(chat.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
    await chat.get('[data-testid=chat-allow]').trigger('click')
    await flushPromises()

    expect(chatApi.answerPermission).toHaveBeenCalledWith(CHAT_ID, 'req-1', true)
    expect(chat.get('[data-testid=chat-allow]').attributes('disabled')).toBeDefined()

    ingest({ seq: 2, type: 'permissionResolved', requestId: 'req-1', allowed: true })
    await flushPromises()
    expect(chat.find('[data-testid=chat-approval]').exists()).toBe(false)
    expect(chat.get('[data-testid=chat-input]').attributes('disabled')).toBeUndefined()
    chat.unmount()
  })

  it('posts Deny, and shows a failed answer on the card', async () => {
    chatApi.answerPermission.mockRejectedValueOnce(new Error('Rekall is down'))
    openChat()
    ingest({ seq: 1, type: 'permissionRequested', requestId: 'req-1', toolUseId: 'u1', name: 'Bash', input: BASH })
    const chat = mountWindow()

    await chat.get('[data-testid=chat-deny]').trigger('click')
    await flushPromises()
    expect(chat.get('[data-testid=chat-approval-error]').text()).toBe('Rekall is down')
    expect(chat.get('[data-testid=chat-deny]').attributes('disabled')).toBeUndefined()

    await chat.get('[data-testid=chat-deny]').trigger('click')
    await flushPromises()
    expect(chatApi.answerPermission).toHaveBeenLastCalledWith(CHAT_ID, 'req-1', false)
    chat.unmount()
  })

  it('disables the composer with its reason once the session ended', async () => {
    openChat()
    const chat = mountWindow()
    useTerminalStore().markEnded(CHAT_ID)
    await flushPromises()

    expect(chat.get('[data-testid=chat-status]').text()).toBe('ended')
    expect(chat.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
    expect(chat.get('[data-testid=chat-hint]').text()).toBe('Session ended')
    chat.unmount()
  })

  it('moves focus to the composer when it opens, and back to it after an answer', async () => {
    openChat()
    const chat = mountWindow()
    await flushPromises()

    expect(document.activeElement).toBe(chat.get('[data-testid=chat-input]').element)
    chat.unmount()
  })

  it('returns focus to the console on Escape without hiding the window', async () => {
    document.body.insertAdjacentHTML('beforeend', '<div id="note" tabindex="-1"></div>')
    openChat()
    const chat = mountWindow()
    await flushPromises()

    await chat.get('[data-testid=chat-window]').trigger('keydown', { key: 'Escape' })

    expect(document.activeElement).toBe(document.getElementById('note'))
    expect(useChatDockStore().hidden).toBe(false)
    chat.unmount()
  })

  it('resizes from the keyboard by 16 px, 64 with Shift, and remembers the width', async () => {
    openChat()
    const dock = useChatDockStore()
    dock.observe(1600, true)
    const chat = mountWindow()
    const handle = chat.get('[role=separator]')

    await handle.trigger('keydown', { key: 'ArrowLeft' })
    expect(dock.width).toBe(456)
    await handle.trigger('keydown', { key: 'ArrowRight', shiftKey: true })
    expect(dock.width).toBe(392)

    expect(storage.get('rekall.chat.width')).toBe('392')
    expect(chat.get('[data-testid=chat-window]').attributes('style')).toContain('width: 392px')
    chat.unmount()
  })

  it('hides itself from its header and expands to the full pane', async () => {
    openChat()
    const dock = useChatDockStore()
    const chat = mountWindow()

    await chat.get('[data-testid=chat-expand]').trigger('click')
    expect(chat.get('[data-testid=chat-window]').attributes('data-mode')).toBe('full')
    expect(chat.get('[data-testid=chat-expand]').text()).toBe('Restore')

    await chat.get('[data-testid=chat-hide]').trigger('click')
    expect(dock.hidden).toBe(true)
    chat.unmount()
  })

  it('stops the whole session only after a confirmation, then closes it', async () => {
    terminalApi.closeTerminal.mockResolvedValue(undefined)
    openChat()
    const chat = mountWindow()

    await chat.get('[data-testid=chat-stop-session]').trigger('click')
    expect(terminalApi.closeTerminal).not.toHaveBeenCalled()
    const confirm = Array.from(document.querySelectorAll('button')).find((button) => button.textContent?.trim() === 'Stop session' && button.className.includes('key-danger') && button.closest('[role=alertdialog]'))
    confirm?.click()
    await flushPromises()

    expect(terminalApi.closeTerminal).toHaveBeenCalledWith(CHAT_ID)
    expect(useChatStore().activeChatId).toBeNull()
    chat.unmount()
  })

  it('shows the way in when the task has no chat session, and opens one as a chat', async () => {
    const opened = chatSession()
    terminalApi.openTerminal.mockResolvedValue(opened)
    const chat = mountWindow()

    expect(chat.text()).toContain('No chat session on this task')
    await chat.get('[data-testid=chat-open]').trigger('click')
    await flushPromises()

    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ kind: 'CHAT' }))
    expect(useChatStore().activeChatId).toBe(CHAT_ID)
    expect(useTerminalStore().activeTerminalId).toBeNull()
    chat.unmount()
  })

  it('asks for the project folder first when there is none', () => {
    consoleFake.selectedTask = task({ projectRepoFolder: null })
    const chat = mountWindow()

    expect(chat.find('[data-testid=chat-open]').exists()).toBe(false)
    expect(chat.find('[data-testid=chat-set-folder]').exists()).toBe(true)
    chat.unmount()
  })

  it('follows the task: selecting one with its own live chat session switches to it', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    terminals.upsert(chatSession({ id: 'chat-2' as never, taskId: 't-2' as TaskId }))
    useChatStore().select(CHAT_ID)
    const chat = mountWindow()

    useChatStore().showTaskSession('t-2' as TaskId)
    await flushPromises()

    expect(useChatStore().activeChatId).toBe('chat-2')
    chat.unmount()
  })
})
