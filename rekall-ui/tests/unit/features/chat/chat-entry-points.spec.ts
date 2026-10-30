import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { effectScope, nextTick } from 'vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import PlanHereButton from '@/features/claude/components/PlanHereButton.vue'
import { preferredLaunchKind, preferredSessionKind, setPreferredSessionKind } from '@/common/config/claude-launch'
import { useRevealOpenedChat } from '@/features/chat/composables/useRevealOpenedChat'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { useConsoleStore } from '@/stores/console.store'
import { useDiagramStore } from '@/features/diagram/store/diagram.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import type { ProjectId, TaskId } from '@/model/branded'
import { CHAT_ID, chatSession } from './support'
import { installStorage } from '../../support/memoryStorage'

const chatApi = { sendMessage: vi.fn() }
const terminalApi = { fetchTerminals: vi.fn(), openTerminal: vi.fn(), closeTerminal: vi.fn(), sendTerminalInput: vi.fn() }
const push = vi.fn()

vi.mock('@/features/chat/api/chat.api', () => ({
  sendMessage: (...a: unknown[]) => chatApi.sendMessage(...a),
  interrupt: vi.fn(),
  answerPermission: vi.fn()
}))

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => terminalApi.fetchTerminals(...a),
  openTerminal: (...a: unknown[]) => terminalApi.openTerminal(...a),
  closeTerminal: (...a: unknown[]) => terminalApi.closeTerminal(...a),
  sendTerminalInput: (...a: unknown[]) => terminalApi.sendTerminalInput(...a)
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({ currentRoute: { value: { name: 'console' } }, push })
}))

vi.mock('@/features/vulnerability/api/vulnerability.api', () => ({}))
vi.mock('@/features/vulnerability/composables/useVulnerabilityStream', () => ({ useVulnerabilityStream: vi.fn() }))

const TASK = 't-1' as TaskId
const ANCHOR = 'project:vega task:report-builder'

function chooseChatView(): void {
  setPreferredSessionKind('chat')
}

describe('the preferred session view', () => {
  beforeEach(() => void installStorage())

  it('is the terminal until the chat is chosen, and remembers the choice', () => {
    expect(preferredSessionKind()).toBe('terminal')
    expect(preferredLaunchKind()).toBe('TERMINAL')

    chooseChatView()

    expect(preferredSessionKind()).toBe('chat')
    expect(preferredLaunchKind()).toBe('CHAT')
  })

  it('ignores a stored value it does not know, and a browser that refuses storage', () => {
    installStorage({ getItem: () => 'holodeck' })
    expect(preferredSessionKind()).toBe('terminal')

    installStorage({
      getItem: () => {
        throw new Error('blocked')
      }
    })
    expect(preferredSessionKind()).toBe('terminal')
  })
})

describe('the entry points', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    Object.values(chatApi).forEach((mock) => mock.mockReset().mockResolvedValue(undefined))
    Object.values(terminalApi).forEach((mock) => mock.mockReset())
    push.mockReset()
  })

  it('Run here asks for a terminal by default and for a chat when the chat view is chosen', async () => {
    terminalApi.openTerminal.mockResolvedValue(chatSession({ kind: 'TERMINAL', id: 'term-1' as never }))
    useConsoleStore().releaseOnRun = vi.fn().mockResolvedValue(null)
    const button = mount(OpenTerminalButton, { props: { taskId: TASK, folder: '/code/vega' } })

    await button.get('[data-testid=open-terminal]').trigger('click')
    await flushPromises()
    expect(terminalApi.openTerminal).toHaveBeenLastCalledWith({ taskId: TASK }, expect.objectContaining({ kind: 'TERMINAL' }))
    button.unmount()

    chooseChatView()
    terminalApi.openTerminal.mockResolvedValue(chatSession())
    const chatButton = mount(OpenTerminalButton, { props: { taskId: TASK, folder: '/code/vega' } })
    await chatButton.get('[data-testid=open-terminal]').trigger('click')
    await flushPromises()

    expect(terminalApi.openTerminal).toHaveBeenLastCalledWith({ taskId: TASK }, expect.objectContaining({ kind: 'CHAT' }))
    expect(useTerminalStore().activeTerminalId).toBe('term-1')
    chatButton.unmount()
  })

  it('Plan here opens a chat session without opening the terminal pane', async () => {
    chooseChatView()
    terminalApi.openTerminal.mockResolvedValue(chatSession())
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')
    const button = mount(PlanHereButton, { props: { taskId: TASK, anchor: ANCHOR, folder: '/code/vega' }, global: { stubs: { AppConfirm: true } } })

    await button.get('[data-testid=plan-here]').trigger('click')
    await flushPromises()

    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN', kind: 'CHAT' }))
    expect(openPane).not.toHaveBeenCalled()
    button.unmount()
  })

  it('Plan, Plan a flow and Replan on a live chat session send the line as a message', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    const input = { skipPermissions: false }

    await terminals.planForTask(TASK, ANCHOR, input, 'PLAN')
    await terminals.planForTask(TASK, ANCHOR, input, 'PLAN_FLOW')
    await terminals.planForTask(TASK, ANCHOR, input, 'REPLAN')

    expect(chatApi.sendMessage.mock.calls).toEqual([
      [CHAT_ID, `/rk ${ANCHOR} plan`],
      [CHAT_ID, `/rk ${ANCHOR} plan flow`],
      [CHAT_ID, `/rk ${ANCHOR} replan`]
    ])
    expect(terminalApi.sendTerminalInput).not.toHaveBeenCalled()
    expect(terminalApi.openTerminal).not.toHaveBeenCalled()
  })

  it('Generate on a project asks for the chosen kind of session', async () => {
    chooseChatView()
    terminalApi.openTerminal.mockResolvedValue(chatSession({ taskId: null }))

    await useDiagramStore().generate({ projectId: 'p-1' as ProjectId, taskId: null, request: 'checkout', kinds: ['sequence'], walkthrough: false })

    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ projectId: 'p-1' }, expect.objectContaining({ mode: 'GENERATE', kind: 'CHAT' }))
  })

  it('the Security actions ask for the chosen kind of session', async () => {
    chooseChatView()
    terminalApi.openTerminal.mockResolvedValue(chatSession({ taskId: null }))

    await useVulnerabilityStore().collectInventory('p-1' as ProjectId)
    await useVulnerabilityStore().assessAll('p-1' as ProjectId)

    expect(terminalApi.openTerminal).toHaveBeenNthCalledWith(1, { projectId: 'p-1' }, expect.objectContaining({ mode: 'INVENTORY', kind: 'CHAT' }))
    expect(terminalApi.openTerminal).toHaveBeenNthCalledWith(2, { projectId: 'p-1' }, expect.objectContaining({ mode: 'ASSESS_ALL', kind: 'CHAT' }))
  })
})

describe('showing a chat session', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    terminalApi.openTerminal.mockReset()
    push.mockReset()
  })

  it('brings a chat session to the window as soon as one is opened, and leaves terminals to their pane', async () => {
    const scope = effectScope()
    scope.run(() => useRevealOpenedChat())
    const terminals = useTerminalStore()
    const chat = useChatStore()
    useChatDockStore().hide()

    terminalApi.openTerminal.mockResolvedValue(chatSession({ kind: 'TERMINAL', id: 'term-1' as never }))
    await terminals.openForTask(TASK, { skipPermissions: false })
    await nextTick()
    expect(chat.activeChatId).toBeNull()
    expect(terminals.activeTerminalId).toBe('term-1')

    terminalApi.openTerminal.mockResolvedValue(chatSession())
    await terminals.openForTask(TASK, { skipPermissions: false, kind: 'CHAT' })
    await nextTick()

    expect(chat.activeChatId).toBe(CHAT_ID)
    expect(useChatDockStore().hidden).toBe(false)
    expect(terminals.activeTerminalId).toBe('term-1')
    scope.stop()
  })

  it('shows a chat session from the dock in the window on its task, not in the terminal pane', () => {
    const console = useConsoleStore()
    const selectTask = vi.spyOn(console, 'selectTask')
    const openPane = vi.spyOn(console, 'openTerminal')
    const open = effectScope().run(() => useOpenSession())!

    open(chatSession())

    expect(selectTask).toHaveBeenCalledWith(TASK)
    expect(useChatStore().activeChatId).toBe(CHAT_ID)
    expect(openPane).not.toHaveBeenCalled()
  })
})

