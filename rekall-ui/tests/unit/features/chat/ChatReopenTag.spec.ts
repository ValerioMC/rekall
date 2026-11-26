import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import ChatReopenTag from '@/features/chat/components/ChatReopenTag.vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { TaskId } from '@/model/branded'
import { useChatStore } from '@/features/chat/store/chat.store'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import { CHAT_ID, chatSession } from './support'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

const TASK = 't-1' as TaskId

function prepare(options: { hidden: boolean; withSession: boolean }): void {
  const dock = useChatDockStore()
  if (options.hidden) dock.hide()
  else dock.open()
  useConsoleStore().selectedTaskId = TASK
  useTerminalStore().terminals = options.withSession ? [chatSession()] : []
}

function ingest(event: ChatEvent): void {
  useChatStore().ingest(CHAT_ID, event)
}

const ASK: ChatEvent = {
  seq: 2,
  type: 'permissionRequested',
  requestId: 'req-1',
  toolUseId: 'u1',
  name: 'Bash',
  input: { kind: 'bash', command: 'ls', description: null }
} as unknown as ChatEvent

describe('ChatReopenTag', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('opens the chat window on the selected task when pressed', async () => {
    prepare({ hidden: true, withSession: true })
    const tag = mount(ChatReopenTag)

    await tag.get('[data-testid=chat-reopen-tag]').trigger('click')

    const dock = useChatDockStore()
    expect(dock.hidden).toBe(false)
    expect(dock.requestedFor).toBe(TASK)
  })

  it('is absent while the chat window is shown', () => {
    prepare({ hidden: false, withSession: true })

    expect(mount(ChatReopenTag).find('[data-testid=chat-reopen-tag]').exists()).toBe(false)
  })

  it('is absent when the selected task has no chat session', () => {
    prepare({ hidden: true, withSession: false })

    expect(mount(ChatReopenTag).find('[data-testid=chat-reopen-tag]').exists()).toBe(false)
  })

  it('names an idle session and shows its still glyph', () => {
    prepare({ hidden: true, withSession: true })

    const tag = mount(ChatReopenTag).get('[data-testid=chat-reopen-tag]')

    expect(tag.attributes('data-status')).toBe('idle')
    expect(tag.attributes('aria-label')).toBe('Open Claude Code Chat, idle')
    expect(tag.find('[data-testid=task-activity-mark]').exists()).toBe(false)
  })

  it('shows the moving bars while Claude is working', () => {
    prepare({ hidden: true, withSession: true })
    ingest({ seq: 1, type: 'userMessage', text: 'go' })

    const tag = mount(ChatReopenTag).get('[data-testid=chat-reopen-tag]')

    expect(tag.attributes('data-status')).toBe('working')
    expect(tag.attributes('aria-label')).toBe('Open Claude Code Chat, working')
    expect(tag.get('[data-testid=task-activity-mark]').attributes('data-activity')).toBe('WORKING')
  })

  it('becomes the amber "Needs your answer" pill while the session waits on a permission', () => {
    prepare({ hidden: true, withSession: true })
    ingest({ seq: 1, type: 'userMessage', text: 'go' })
    ingest(ASK)

    const tag = mount(ChatReopenTag).get('[data-testid=chat-reopen-tag]')

    expect(tag.attributes('data-status')).toBe('waiting')
    expect(tag.attributes('aria-label')).toBe('Open Claude Code Chat, waiting for your answer')
    expect(tag.get('[data-testid=chat-reopen-label]').text()).toBe('Needs your answer')
    expect(tag.classes()).toContain('bg-accent')
    expect(tag.find('[data-testid=chat-reopen-ping]').exists()).toBe(true)
  })

  it('shows a waiting session whose permission flag came from the stored list', () => {
    prepare({ hidden: true, withSession: true })
    useTerminalStore().terminals = [chatSession({ awaitingPermission: true })]

    expect(mount(ChatReopenTag).get('[data-testid=chat-reopen-tag]').attributes('data-status')).toBe('waiting')
  })

  it('names an ended session apart from the others', () => {
    prepare({ hidden: true, withSession: true })
    useChatStore().select(CHAT_ID)
    ingest({ seq: 1, type: 'ended', exitCode: 0, detail: '' })

    const tag = mount(ChatReopenTag).get('[data-testid=chat-reopen-tag]')

    expect(tag.attributes('data-status')).toBe('ended')
    expect(tag.attributes('aria-label')).toBe('Open Claude Code Chat, ended')
    expect(tag.classes()).toContain('border-dashed')
  })
})
