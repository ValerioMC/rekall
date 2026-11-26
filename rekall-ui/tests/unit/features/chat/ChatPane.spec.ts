import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import ChatPane from '@/features/chat/components/ChatPane.vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useTaskProviderStore } from '@/features/claude/store/taskProvider.store'
import { setCodexEffort, setCodexModel, setPreferredProvider } from '@/common/config/session-launch'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'
import { CHAT_ID, MarkdownStub, chatSession } from './support'
import { taskFixture } from '../../support/taskFixture'
import { installStorage } from '../../support/memoryStorage'

const terminalApi = { fetchTerminals: vi.fn(), openTerminal: vi.fn(), closeTerminal: vi.fn() }

const chatApi = { sendMessage: vi.fn(), interrupt: vi.fn(), answerPermission: vi.fn() }

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

const TASK = 't-1' as TaskId
const OTHER = 't-2' as TaskId
const OTHER_CHAT = 'chat-2' as TerminalId
const LOOSE_CHAT = 'chat-9' as TerminalId

const Page = { render: () => null }

let pinia: Pinia
let router: Router
let pane: VueWrapper | null = null

function otherChat(): Terminal {
  return chatSession({ id: OTHER_CHAT, taskId: OTHER, taskTitle: 'Retry policy', anchors: 'task:t-2' })
}

function looseChat(): Terminal {
  return chatSession({ id: LOOSE_CHAT, taskId: null, taskCode: null, taskTitle: null, anchors: 'project:vega' })
}

/** The server's session list, which the store holds from startup and the view reads again. */
function sessions(...listed: Terminal[]): void {
  useTerminalStore().terminals = listed
  terminalApi.fetchTerminals.mockResolvedValue(listed)
}

async function mountPane(): Promise<VueWrapper> {
  await router.push('/console')
  pane = mount(ChatPane, {
    attachTo: document.body,
    global: { plugins: [pinia, router], stubs: { AppMarkdownEditor: MarkdownStub, TerminalConnectionChips: true } }
  })
  await flushPromises()
  return pane
}

function showChatOf(taskId: TaskId): void {
  useConsoleStore().selectTask(taskId)
  useConsoleStore().openChat()
}

describe('the full-size chat', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    installStorage()
    Object.values(terminalApi).forEach((mock) => mock.mockReset())
    terminalApi.closeTerminal.mockResolvedValue(undefined)
    Object.values(chatApi).forEach((mock) => mock.mockReset().mockResolvedValue(undefined))
    useChatStore().noteConnection({ connected: true, reconnecting: false })
    useConsoleStore().tasks = [
      { ...taskFixture(TASK), title: 'Report builder', projectRepoFolder: '/code/vega' },
      { ...taskFixture(OTHER), title: 'Retry policy' }
    ]
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/console', name: 'console', component: Page },
        { path: '/task/:id/:tab?', name: 'task', component: Page },
        { path: '/projects/:id?', name: 'project-detail', component: Page }
      ]
    })
  })

  afterEach(() => {
    pane?.unmount()
    pane = null
    document.body.innerHTML = ''
  })

  it('heads the task\'s chat with its title, state, provider and anchors, over the conversation', async () => {
    sessions(chatSession())
    showChatOf(TASK)

    const view = await mountPane()

    expect(view.get('[data-testid=chat-task-link]').text()).toBe('Report builder')
    expect(view.get('[data-testid=chat-task-link]').attributes('href')).toBe('/task/t-1/description')
    expect(view.get('[data-testid=chat-status]').text()).toBe('idle')
    expect(view.get('[data-testid=chat-header]').text()).toContain('Claude Code chat')
    expect(view.get('[data-testid=chat-header]').text()).toContain('project:vega task:report-builder')
    expect(view.find('[data-testid=chat-input]').exists()).toBe(true)
  })

  it('lists the session\'s step, permissions, context and cost beside the conversation', async () => {
    useConsoleStore().steps = [{ id: 's1' as TaskStepId, taskId: TASK, title: 'Write the CSV file', state: 'OPEN', startsSession: false, position: 0 } as TaskStep]
    sessions(chatSession({ stepId: 's1' as TaskStepId, model: 'opus' }))
    showChatOf(TASK)
    const view = await mountPane()

    useChatStore().ingest(CHAT_ID, {
      seq: 1,
      type: 'turnFinished',
      usage: { inputTokens: 1, cacheCreationInputTokens: 0, cacheReadInputTokens: 0, outputTokens: 1 },
      durationMs: 10,
      outcome: 'completed',
      costUsd: 0.42,
      context: { usedTokens: 68_000, windowTokens: 200_000 }
    })
    await flushPromises()

    const facts = view.get('[data-testid=chat-facts]').text()
    expect(facts).toContain('Step 1 · Write the CSV file')
    expect(facts).toContain('opus')
    expect(facts).toContain('permissions on')
    expect(view.get('[data-testid=chat-readout-context]').text()).toBe('context 34%')
    expect(view.get('[data-testid=chat-context-tokens]').text()).toBe('68,000 of 200,000 tokens')
    expect(view.get('[data-testid=chat-readout-cost]').text()).toBe('$0.42')
  })

  it('reads the session list when it appears and again when the task changes, moving to that task\'s chat', async () => {
    sessions(chatSession(), otherChat())
    showChatOf(TASK)
    const view = await mountPane()
    expect(terminalApi.fetchTerminals).toHaveBeenCalledTimes(1)

    showChatOf(OTHER)
    await flushPromises()

    expect(terminalApi.fetchTerminals).toHaveBeenCalledTimes(2)
    expect(view.get('[data-testid=chat-task-link]').text()).toBe('Retry policy')
    expect(useChatStore().activeChatId).toBe(OTHER_CHAT)
  })

  it('offers Run here on a task with no chat, never showing a chat of no task in its place', async () => {
    sessions(looseChat())
    useChatStore().select(LOOSE_CHAT)
    showChatOf(TASK)

    const view = await mountPane()

    expect(view.get('[data-testid=chat-empty]').text()).toContain('No chat session on this task')
    expect(view.get('[data-testid=chat-task-link]').text()).toBe('Report builder')
    expect(view.find('[data-testid=chat-status]').exists()).toBe(false)
    expect(view.find('[data-testid=chat-stop-session]').exists()).toBe(false)
  })

  it('shows the chat that Run here opens', async () => {
    sessions()
    showChatOf(TASK)
    const view = await mountPane()
    terminalApi.openTerminal.mockResolvedValue(chatSession())

    await view.get('[data-testid=chat-open]').trigger('click')
    await flushPromises()

    expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ kind: 'CHAT' }))
    expect(view.find('[data-testid=chat-input]').exists()).toBe(true)
  })

  it('shows the terminal view of the task when Run here gets back a terminal', async () => {
    sessions()
    showChatOf(TASK)
    const view = await mountPane()
    terminalApi.openTerminal.mockResolvedValue(chatSession({ id: 'term-1' as TerminalId, kind: 'TERMINAL' }))

    await view.get('[data-testid=chat-open]').trigger('click')
    await flushPromises()

    expect(useConsoleStore().paneFocus).toBe('terminal')
    expect(useTerminalStore().activeTerminalId).toBe('term-1')
  })

  it('shows the empty view once the list is read and an ended chat has left it', async () => {
    sessions(chatSession())
    terminalApi.fetchTerminals.mockResolvedValue([])
    showChatOf(TASK)

    const view = await mountPane()

    expect(view.find('[data-testid=chat-empty]').exists()).toBe(true)
  })

  it('asks for the project folder first when there is none', async () => {
    sessions()
    showChatOf(OTHER)

    const view = await mountPane()

    expect(view.find('[data-testid=chat-open]').exists()).toBe(false)
    expect(view.find('[data-testid=chat-set-folder]').exists()).toBe(true)
  })

  it('moves to another live chat from the switcher, selecting its task', async () => {
    sessions(chatSession(), otherChat(), chatSession({ id: 'term-1' as TerminalId, taskId: 't-3' as TaskId, kind: 'TERMINAL' }))
    showChatOf(TASK)
    const view = await mountPane()

    expect(view.get('[data-testid=other-sessions]').text()).toBe('Other 1 session')
    await view.get('[data-testid=other-sessions]').trigger('click')
    await view.get('[data-testid=other-sessions-row]').trigger('click')
    await flushPromises()

    expect(useConsoleStore().selectedTaskId).toBe(OTHER)
    expect(useChatStore().activeChatId).toBe(OTHER_CHAT)
  })

  it('shows a chat on no task under its project\'s name, which leads nowhere', async () => {
    sessions(looseChat())
    useConsoleStore().openProjectChat(LOOSE_CHAT)

    const view = await mountPane()

    expect(view.get('[data-testid=chat-header] h2').text()).toBe('Project vega')
    expect(view.find('[data-testid=chat-task-link]').exists()).toBe(false)
    expect(view.find('[data-testid=chat-input]').exists()).toBe(true)
  })

  it('says a chat on no task has closed once the list no longer holds it', async () => {
    sessions()
    useConsoleStore().openProjectChat(LOOSE_CHAT)

    const view = await mountPane()

    expect(view.get('[data-testid=chat-closed]').text()).toContain('This chat session has closed')
  })

  async function confirmStop(view: VueWrapper): Promise<void> {
    await view.get('[data-testid=chat-stop-session]').trigger('click')
    expect(terminalApi.closeTerminal).not.toHaveBeenCalled()
    Array.from(document.querySelectorAll('button'))
      .find((button) => button.textContent?.trim() === 'Stop session' && button.closest('[role=alertdialog]'))
      ?.click()
    await flushPromises()
  }

  it('stays on the task with the empty view once its session is stopped', async () => {
    sessions(chatSession())
    showChatOf(TASK)
    const view = await mountPane()

    await confirmStop(view)

    expect(terminalApi.closeTerminal).toHaveBeenCalledWith(CHAT_ID)
    expect(view.find('[data-testid=chat-empty]').exists()).toBe(true)
    expect(router.currentRoute.value.name).toBe('console')
  })

  it('goes to the project\'s page once a chat on no task is stopped', async () => {
    sessions(looseChat())
    useConsoleStore().openProjectChat(LOOSE_CHAT)
    const view = await mountPane()

    await confirmStop(view)

    expect(terminalApi.closeTerminal).toHaveBeenCalledWith(LOOSE_CHAT)
    expect(router.currentRoute.value.fullPath).toBe('/projects/p-1')
  })

  describe('its conversation', () => {
    const BASH = { tool: 'bash', command: 'ls', description: null } as const

    async function mountConversation(): Promise<VueWrapper> {
      sessions(chatSession())
      showChatOf(TASK)
      return mountPane()
    }

    function ingest(event: ChatEvent): void {
      useChatStore().ingest(CHAT_ID, event)
    }

    function input(view: VueWrapper): HTMLTextAreaElement {
      return view.get('[data-testid=chat-input]').element as HTMLTextAreaElement
    }

    it('reads reconnecting while the followed stream is retried', async () => {
      const view = await mountConversation()

      useChatStore().noteConnection({ connected: false, reconnecting: true })
      await flushPromises()

      expect(view.get('[data-testid=chat-status]').text()).toBe('reconnecting')
      expect(view.find('[data-testid=chat-reconnecting]').exists()).toBe(true)
    })

    it('puts the keyboard in the message field when it opens', async () => {
      const view = await mountConversation()

      expect(document.activeElement).toBe(input(view))
    })

    it('sends the typed message through the store and clears the field', async () => {
      const view = await mountConversation()

      await view.get('[data-testid=chat-input]').setValue('hello')
      await view.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })
      await flushPromises()

      expect(chatApi.sendMessage).toHaveBeenCalledWith(CHAT_ID, 'hello')
      expect(input(view).value).toBe('')
    })

    it('keeps the text and says why when the send fails', async () => {
      chatApi.sendMessage.mockRejectedValue(new Error('Session is gone'))
      const view = await mountConversation()

      await view.get('[data-testid=chat-input]').setValue('hello')
      await view.get('[data-testid=chat-send]').trigger('click')
      await flushPromises()

      expect(input(view).value).toBe('hello')
      expect(view.get('[data-testid=chat-send-error]').text()).toBe('Session is gone')
    })

    it('stops the running turn through interrupt and shows Stopping until the turn ends', async () => {
      const view = await mountConversation()
      ingest({ seq: 1, type: 'userMessage', text: 'go' })
      await flushPromises()

      await view.get('[data-testid=chat-stop]').trigger('click')
      await flushPromises()
      expect(chatApi.interrupt).toHaveBeenCalledWith(CHAT_ID)
      expect(view.get('[data-testid=chat-stop]').text()).toBe('Stopping…')

      ingest({
        seq: 2,
        type: 'turnFinished',
        usage: { inputTokens: 1, cacheCreationInputTokens: 0, cacheReadInputTokens: 0, outputTokens: 1 },
        durationMs: 10,
        outcome: 'interrupted',
        costUsd: null,
        context: null
      })
      await flushPromises()
      expect(view.find('[data-testid=chat-stop]').exists()).toBe(false)
    })

    it('posts Allow, keeps the card disabled until the resolution arrives, then clears it', async () => {
      const view = await mountConversation()
      ingest({ seq: 1, type: 'permissionRequested', requestId: 'req-1', toolUseId: 'u1', name: 'Bash', input: BASH })
      await flushPromises()

      expect(view.get('[data-testid=chat-status]').text()).toBe('waiting for approval')
      expect(view.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
      await view.get('[data-testid=chat-allow]').trigger('click')
      await flushPromises()

      expect(chatApi.answerPermission).toHaveBeenCalledWith(CHAT_ID, 'req-1', true)
      expect(view.get('[data-testid=chat-allow]').attributes('disabled')).toBeDefined()

      ingest({ seq: 2, type: 'permissionResolved', requestId: 'req-1', allowed: true })
      await flushPromises()
      expect(view.find('[data-testid=chat-approval]').exists()).toBe(false)
      expect(view.get('[data-testid=chat-input]').attributes('disabled')).toBeUndefined()
    })

    it('posts Deny, and shows a failed answer on the card', async () => {
      chatApi.answerPermission.mockRejectedValueOnce(new Error('Rekall is down'))
      const view = await mountConversation()
      ingest({ seq: 1, type: 'permissionRequested', requestId: 'req-1', toolUseId: 'u1', name: 'Bash', input: BASH })
      await flushPromises()

      await view.get('[data-testid=chat-deny]').trigger('click')
      await flushPromises()
      expect(view.get('[data-testid=chat-approval-error]').text()).toBe('Rekall is down')
      expect(view.get('[data-testid=chat-deny]').attributes('disabled')).toBeUndefined()

      await view.get('[data-testid=chat-deny]').trigger('click')
      await flushPromises()
      expect(chatApi.answerPermission).toHaveBeenLastCalledWith(CHAT_ID, 'req-1', false)
    })

    it('disables the message field with its reason once the session ended', async () => {
      const view = await mountConversation()

      useTerminalStore().markEnded(CHAT_ID)
      await flushPromises()

      expect(view.get('[data-testid=chat-status]').text()).toBe('ended')
      expect(view.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
      expect(view.get('[data-testid=chat-hint]').text()).toBe('Session ended')
    })
  })

  describe('Run here on a task with no chat', () => {
    async function pressRunHere(view: VueWrapper): Promise<void> {
      await view.get('[data-testid=chat-open]').trigger('click')
      await flushPromises()
    }

    it('opens with the preferred Codex options and shows the actual provider', async () => {
      setPreferredProvider('CODEX')
      setCodexModel('gpt-test')
      setCodexEffort('high')
      sessions()
      showChatOf(TASK)
      const view = await mountPane()
      terminalApi.openTerminal.mockResolvedValue(chatSession({ provider: 'CODEX' }))

      await pressRunHere(view)

      expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({
        provider: 'CODEX', kind: 'CHAT', model: 'gpt-test', effort: 'high', skipPermissions: false
      }))
      expect(view.get('[data-testid=chat-header]').text()).toContain('Codex chat')
      expect(view.get('[data-testid=chat-input]').attributes('aria-label')).toBe('Message to Codex')
    })

    it('opens with the provider the task was switched to', async () => {
      useTaskProviderStore().choose(TASK, 'CODEX')
      sessions()
      showChatOf(TASK)
      const view = await mountPane()
      terminalApi.openTerminal.mockResolvedValue(chatSession({ provider: 'CODEX' }))

      await pressRunHere(view)

      expect(terminalApi.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ provider: 'CODEX' }))
    })
  })
})
