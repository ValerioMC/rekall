import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import ChatReopenTag from '@/features/chat/components/ChatReopenTag.vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { TaskId } from '@/model/branded'
import { chatSession } from './support'
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
})
