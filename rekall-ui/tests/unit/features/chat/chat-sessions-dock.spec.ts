import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TerminalSessionsPanel from '@/components/shell/TerminalSessionsPanel.vue'
import ShellDock from '@/components/shell/ShellDock.vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { CHAT_ID, chatSession } from './support'
import { installStorage } from '../../support/memoryStorage'

const route = { name: 'console' }
const terminalApi = { closeTerminal: vi.fn() }

vi.mock('vue-router', () => ({
  useRoute: () => route,
  useRouter: () => ({ currentRoute: { value: route }, push: vi.fn() })
}))

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: (...a: unknown[]) => terminalApi.closeTerminal(...a),
  sendTerminalInput: vi.fn()
}))

class SilentResizeObserver implements ResizeObserver {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

describe('chat sessions in the sessions dock', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    terminalApi.closeTerminal.mockReset().mockResolvedValue(undefined)
    vi.stubGlobal('ResizeObserver', SilentResizeObserver)
  })

  it('lists a chat beside a terminal with a badge, and marks the one waiting for approval', () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    terminals.upsert(chatSession({ id: 'chat-2' as never, awaitingPermission: true }))
    terminals.upsert(chatSession({ id: 'term-1' as never, kind: 'TERMINAL' }))
    const panel = mount(TerminalSessionsPanel, { global: { stubs: { TerminalConnectionChips: true } } })

    const badges = panel.findAll('[data-testid=terminal-dock-kind]').map((badge) => badge.text())

    expect(panel.findAll('[data-testid=terminal-dock-row]')).toHaveLength(3)
    expect(badges.sort()).toEqual(['approve', 'chat'])
  })

  it('opens a chat row in the chat window and closes it through the shared close call', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    const panel = mount(TerminalSessionsPanel, { global: { stubs: { TerminalConnectionChips: true } } })

    await panel.get('[data-testid=terminal-dock-jump]').trigger('click')
    expect(useChatStore().activeChatId).toBe(CHAT_ID)

    await panel.get('[data-testid=terminal-dock-stop]').trigger('click')
    await Promise.resolve()
    expect(terminalApi.closeTerminal).toHaveBeenCalledWith(CHAT_ID)
  })

  it('turns the sessions segment into "approve" while a chat waits, and steps aside while the window is shown', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession({ awaitingPermission: true }))
    const dock = mount(ShellDock, { global: { stubs: { TerminalSessionsPanel: true, RunningTasksPanel: true } } })

    expect(dock.find('[data-testid=terminal-dock-toggle]').exists()).toBe(false)
    useChatDockStore().hide()
    await dock.vm.$nextTick()

    expect(dock.get('[data-testid=terminal-dock-toggle]').text()).toBe('approve')
  })
})
