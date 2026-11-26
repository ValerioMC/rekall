import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TerminalSessionsPanel from '@/components/shell/TerminalSessionsPanel.vue'
import ShellDock from '@/components/shell/ShellDock.vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { TaskId } from '@/model/branded'
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

describe('chat sessions in the sessions dock', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    terminalApi.closeTerminal.mockReset().mockResolvedValue(undefined)
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

  it('groups the sessions working on a task apart from the ones that are not', () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession({ id: 'on-task' as never, taskId: 'task-1' as TaskId }))
    terminals.upsert(chatSession({ id: 'no-task' as never, taskId: null }))
    const panel = mount(TerminalSessionsPanel, { global: { stubs: { TerminalConnectionChips: true } } })

    expect(panel.get('[data-testid=terminal-dock-group-task]').text()).toBe('Working on a task')
    expect(panel.get('[data-testid=terminal-dock-group-project]').text()).toBe('Not on a task')
    expect(panel.findAll('[data-testid=terminal-dock-row]')).toHaveLength(2)
  })

  it('omits a group with no session', () => {
    useTerminalStore().upsert(chatSession({ taskId: null }))
    const panel = mount(TerminalSessionsPanel, { global: { stubs: { TerminalConnectionChips: true } } })

    expect(panel.find('[data-testid=terminal-dock-group-task]').exists()).toBe(false)
    expect(panel.find('[data-testid=terminal-dock-group-project]').exists()).toBe(true)
  })

  it('opens a chat row full-size and closes it through the shared close call', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    const panel = mount(TerminalSessionsPanel, { global: { stubs: { TerminalConnectionChips: true } } })

    await panel.get('[data-testid=terminal-dock-jump]').trigger('click')
    expect(useChatStore().activeChatId).toBe(CHAT_ID)
    expect(useConsoleStore().paneFocus).toBe('chat')

    await panel.get('[data-testid=terminal-dock-stop]').trigger('click')
    await Promise.resolve()
    expect(terminalApi.closeTerminal).toHaveBeenCalledWith(CHAT_ID)
  })

  it('turns the sessions segment into "approve" while a chat waits, on the page of its own task', () => {
    useTerminalStore().upsert(chatSession({ awaitingPermission: true }))
    useConsoleStore().selectedTaskId = 't-1' as TaskId
    route.name = 'task'
    const dock = mount(ShellDock, { global: { stubs: { TerminalSessionsPanel: true, RunningTasksPanel: true } } })

    expect(dock.get('[data-testid=terminal-dock-toggle]').text()).toBe('approve')
  })
})
