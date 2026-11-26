import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { effectScope, nextTick } from 'vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import SessionLaunchConflict from '@/features/claude/components/SessionLaunchConflict.vue'
import { ApiError } from '@/api/client'
import { preferredLaunchKind, preferredSessionKind, setPreferredSessionKind } from '@/common/config/claude-launch'
import { launchChoices, setCodexBypass, setCodexEffort, setCodexModel, setPreferredProvider } from '@/common/config/session-launch'
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

  it.each(['CLAUDE', 'CODEX'] as const)('Plan here opens %s Chat without opening the terminal pane', async (provider) => {
    chooseChatView()
    terminalApi.openTerminal.mockResolvedValue(chatSession({ provider }))
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')
    const button = mount(OpenTerminalButton, { props: { taskId: TASK, folder: '/code/vega' }, attachTo: document.body })

    await button.get(`[data-testid=provider-switch-${provider}]`).trigger('click')
    await button.get('[data-testid=run-menu-arrow]').trigger('click')
    document.body.querySelector<HTMLElement>('[data-testid=plan-here]')?.click()
    await flushPromises()

    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN', kind: 'CHAT', provider }))
    expect(openPane).not.toHaveBeenCalled()
    button.unmount()
  })

  it('keeps the existing chat view when the server reuses it for preparation', async () => {
    const terminals = useTerminalStore()
    terminals.upsert(chatSession())
    terminalApi.openTerminal.mockResolvedValue(chatSession())

    const opened = await terminals.planForTask(TASK, { provider: 'CLAUDE', skipPermissions: false }, 'PLAN_FLOW')

    expect(opened.kind).toBe('CHAT')
    expect(terminals.lastOpened?.id).toBe(CHAT_ID)
    expect(terminals.activeTerminalId).toBeNull()
    expect(terminalApi.openTerminal).toHaveBeenCalledWith(
      { taskId: TASK }, expect.objectContaining({ mode: 'PLAN_FLOW', provider: 'CLAUDE' })
    )
    expect(chatApi.sendMessage).not.toHaveBeenCalled()
    expect(terminalApi.sendTerminalInput).not.toHaveBeenCalled()
  })

  it('opens Codex Refine as a chat with the selected options', async () => {
    chooseChatView()
    setPreferredProvider('CODEX')
    setCodexModel('gpt-test')
    setCodexEffort('high')
    setCodexBypass(true)
    const opened = chatSession({ provider: 'CODEX', mode: 'REFINE' })
    terminalApi.openTerminal.mockResolvedValue(opened)

    expect(await useTerminalStore().planForTask(TASK, launchChoices(), 'REFINE')).toEqual(opened)
    expect(terminalApi.openTerminal).toHaveBeenCalledTimes(1)
    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({
      provider: 'CODEX', kind: 'CHAT', mode: 'REFINE', model: 'gpt-test', effort: 'high', skipPermissions: true
    }))
  })

  it.each([true, false])('asks before retrying an unsupported Codex Chat in Terminal (%s)', async (confirm) => {
    chooseChatView()
    setPreferredProvider('CODEX')
    const error = new ApiError(409, 'Codex Chat could not start. Try updating the Codex CLI.', 'test')
    terminalApi.openTerminal.mockRejectedValueOnce(error)
    terminalApi.openTerminal.mockResolvedValue(chatSession({ provider: 'CODEX', kind: 'TERMINAL' }))
    const terminals = useTerminalStore()
    const dialog = mount(SessionLaunchConflict, { attachTo: document.body, global: { stubs: { Teleport: true } } })
    const input = { ...launchChoices(), model: 'gpt-test', effort: 'high' }
    const opened = terminals.planForTask(TASK, input, 'REFINE').catch((failure: unknown) => failure)
    await flushPromises()

    expect(dialog.text()).toContain(error.message)
    expect(document.activeElement?.textContent?.trim()).toBe('Cancel')
    expect(terminalApi.openTerminal).toHaveBeenCalledTimes(1)
    const label = confirm ? 'Open in Terminal' : 'Cancel'
    await dialog.findAll('button').find((button) => button.text() === label)!.trigger('click')

    if (confirm) {
      expect(await opened).toMatchObject({ provider: 'CODEX', kind: 'TERMINAL' })
      expect(terminalApi.openTerminal).toHaveBeenNthCalledWith(2, { taskId: TASK }, expect.objectContaining({
        ...input, kind: 'TERMINAL', mode: 'REFINE'
      }))
    } else {
      expect(await opened).toBe(error)
      expect(terminalApi.openTerminal).toHaveBeenCalledTimes(1)
    }
    expect(preferredSessionKind()).toBe('chat')
    dialog.unmount()
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

