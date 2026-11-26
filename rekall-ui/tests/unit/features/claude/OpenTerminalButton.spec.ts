import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { setPreferredEffort, setPreferredModel, setPreferredSessionKind, setSkipsPermissions } from '@/common/config/claude-launch'
import { preferredProvider, setCodexEffort, setCodexModel, setPreferredProvider } from '@/common/config/session-launch'
import { useCodexStore } from '@/features/codex/store/codex.store'
import { installStorage } from '../../support/memoryStorage'
import { installBothSessionProviders } from '../../support/sessionProviders'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Terminal } from '@/features/claude/model/terminal'
import { EMPTY_RUN_QUEUE, asRunQueueItemId, type RunQueue, type RunQueueItem } from '@/features/queue/model/runQueue'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

const openTerminal = vi.fn()
const sendTerminalInput = vi.fn()
const releaseOnRun = vi.fn()

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...args: unknown[]) => openTerminal(...args),
  closeTerminal: vi.fn(),
  sendTerminalInput: (...args: unknown[]) => sendTerminalInput(...args)
}))

const push = vi.fn()

vi.mock('vue-router', () => ({
  useRouter: () => ({ currentRoute: { value: { name: 'task' } }, push })
}))

const runTaskInQueue = vi.fn()

vi.mock('@/features/queue/api/runQueue.api', () => ({
  runTaskInQueue: (...args: unknown[]) => runTaskInQueue(...args)
}))

const TASK = 't-1' as TaskId
const STEP = 's-1' as TaskStepId

/** Only what the button reads of a task and a step; the rest of the record plays no part. */
function checklistTask(): Task {
  return { id: TASK, reviewActive: false, reviewState: 'OPEN' } as Task
}

function step(state: TaskStep['state']): TaskStep {
  return { id: STEP, taskId: TASK, state } as TaskStep
}

function terminal(): Terminal {
  return {
    id: 'term-1' as TerminalId,
    projectId: 'p-1' as ProjectId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskCode: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
    provider: 'CLAUDE',
    skipPermissions: true,
    model: null,
    effort: null,
    kind: 'TERMINAL',
    live: true,
    awaitingPermission: false,
    startedAt: '2026-09-22T10:00:00Z',
    lastActivityAt: '2026-09-22T10:00:00Z',
    connections: []
  }
}

function queueItem(taskId: TaskId, position: number, state: RunQueueItem['state']): RunQueueItem {
  return {
    id: asRunQueueItemId(`item-${position}`),
    provider: 'CLAUDE',
    codexOptions: null,
    taskId,
    taskTitle: 'Task',
    taskCode: 'task',
    projectLabel: 'vega',
    anchor: 'project:vega task:task',
    position,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null,
    steps: [],
    waitingFor: []
  }
}

function queueWith(items: RunQueueItem[]): RunQueue {
  return { ...EMPTY_RUN_QUEUE, state: 'RUNNING', items, updatedAt: '2026-09-23T10:00:05Z' }
}

function menuButton(testid: string): HTMLButtonElement {
  return document.body.querySelector<HTMLButtonElement>(`[data-testid="${testid}"]`)!
}

async function switchTo(wrapper: ReturnType<typeof mountButton>, provider: Terminal['provider']): Promise<void> {
  await wrapper.get(`[data-testid="provider-switch-${provider}"]`).trigger('click')
}

async function chooseRunInQueue(
  wrapper: ReturnType<typeof mountButton>,
  provider: Terminal['provider'] = 'CLAUDE'
): Promise<void> {
  await switchTo(wrapper, provider)
  await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
  const entry = menuButton('run-in-queue')
  entry.click()
  entry.click()
  await flushPromises()
}

function menuLabels(): string[] {
  return [...document.body.querySelectorAll('[role="menuitem"]')].map((item) => item.textContent?.trim() ?? '')
}

function mountButton(folder: string | null = '/code/vega', stepId: TaskStepId | null = null) {
  useConsoleStore().releaseOnRun = releaseOnRun
  return mount(OpenTerminalButton, {
    props: { taskId: TASK, folder, stepId },
    attachTo: document.body,
    global: { stubs: { RouterLink: RouterLinkStub } }
  })
}

describe('the Run here button', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
    openTerminal.mockReset()
    releaseOnRun.mockReset().mockResolvedValue(null)
    runTaskInQueue.mockReset()
    push.mockReset()
    vi.useFakeTimers()
  })

  afterEach(() => vi.useRealTimers())

  it('starts the session without switching the console to its terminal', async () => {
    openTerminal.mockResolvedValue(terminal())
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')
    const wrapper = mountButton()

    await wrapper.get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ stepId: null }))
    expect(openPane).not.toHaveBeenCalled()
    expect(push).not.toHaveBeenCalled()
  })

  it('shows the chat it opens full-size on its task, as soon as the server answers', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), kind: 'CHAT' })
    const wrapper = mountButton()

    await wrapper.get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect([useConsoleStore().selectedTaskId, useConsoleStore().paneFocus]).toEqual([TASK, 'chat'])
    expect(useChatStore().activeChatId).toBe('term-1')
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('runs the play mark out, then stamps the amber step seal, then turns the seal of a running step', async () => {
    openTerminal.mockResolvedValue(terminal())
    const wrapper = mountButton()
    const button = wrapper.get('[data-testid="open-terminal"]')

    await button.trigger('click')
    expect(button.attributes('data-phase')).toBe('launching')

    await vi.advanceTimersByTimeAsync(380)
    expect(button.attributes('data-phase')).toBe('done')
    const seal = wrapper.get('.run-seal [data-testid="step-seal"]')
    expect(seal.attributes('data-arrive')).toBe('DONE')
    expect(seal.attributes('data-complete')).toBeUndefined()
    expect(button.classes()).toContain('key-lit')

    await vi.advanceTimersByTimeAsync(1200)
    expect(button.attributes('data-phase')).toBe('working')
    expect(button.classes()).toContain('key-lit')
    expect(wrapper.get('.run-working [data-testid="step-seal"]').attributes('data-state')).toBe('RUNNING')
  })

  /** The server's RUNNING mark may trail the launch; if it never comes, the button is not stuck. */
  it('returns to rest when no session is reported at work after the launch', async () => {
    openTerminal.mockResolvedValue(terminal())
    const button = mountButton().get('[data-testid="open-terminal"]')

    await button.trigger('click')
    await vi.advanceTimersByTimeAsync(380 + 1200 + 8000)

    expect(button.attributes('data-phase')).toBe('idle')
  })

  it('stays working while its step runs, refuses a press, and returns once the step is claimed', async () => {
    const console_ = useConsoleStore()
    console_.tasks = [checklistTask()]
    console_.steps = [step('RUNNING')]
    const wrapper = mountButton('/code/vega', STEP)
    const button = wrapper.get('[data-testid="open-terminal"]')
    expect(button.attributes('data-phase')).toBe('working')

    await button.trigger('click')
    expect(openTerminal).not.toHaveBeenCalled()

    console_.applyStepEvent(TASK, [step('CLAIMED')])
    await flushPromises()
    expect(button.attributes('data-phase')).toBe('idle')
    expect(button.text()).toContain('Run here')
  })

  it('hands the launch over to the step once the server marks it running', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), stepId: STEP })
    const console_ = useConsoleStore()
    console_.tasks = [checklistTask()]
    console_.steps = [step('OPEN')]
    const button = mountButton('/code/vega', STEP).get('[data-testid="open-terminal"]')

    await button.trigger('click')
    console_.applyStepEvent(TASK, [step('RUNNING')])
    await vi.advanceTimersByTimeAsync(380 + 1200)
    expect(button.attributes('data-phase')).toBe('working')

    console_.applyStepEvent(TASK, [step('CLAIMED')])
    await flushPromises()
    expect(button.attributes('data-phase')).toBe('idle')
  })

  it('says nothing will run, and starts no session, when only drafts are left after the claims', async () => {
    const console_ = useConsoleStore()
    console_.tasks = [checklistTask()]
    console_.steps = [step('CLAIMED'), { ...step('DRAFT'), id: 's-2' as TaskStepId }]
    const toast = useToastStore()
    const wrapper = mountButton()

    await wrapper.get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(openTerminal).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toContain('Nothing to run')
    expect(wrapper.get('[data-testid="open-terminal"]').attributes('data-phase')).toBe('idle')
  })

  it('holds an open, turning ring while the server has not answered', async () => {
    let answer: (value: Terminal) => void = () => undefined
    openTerminal.mockReturnValue(new Promise<Terminal>((resolve) => (answer = resolve)))
    const wrapper = mountButton()
    const button = wrapper.get('[data-testid="open-terminal"]')

    await button.trigger('click')
    await vi.advanceTimersByTimeAsync(380)
    expect(button.attributes('data-phase')).toBe('waiting')
    expect(wrapper.find('.run-ring-open').exists()).toBe(true)

    answer(terminal())
    await flushPromises()
    expect(button.attributes('data-phase')).toBe('done')
  })

  it('ignores a second press while a launch is under way', async () => {
    openTerminal.mockResolvedValue(terminal())
    const button = mountButton().get('[data-testid="open-terminal"]')

    await button.trigger('click')
    await button.trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledTimes(1)
  })

  it('goes back to rest when the server refuses', async () => {
    openTerminal.mockRejectedValue(new Error('no folder on disk'))
    const button = mountButton().get('[data-testid="open-terminal"]')

    await button.trigger('click')
    await vi.advanceTimersByTimeAsync(380)

    expect(button.attributes('data-phase')).toBe('idle')
    expect(useToastStore().toasts.map((toast) => toast.message)).toContain('no folder on disk')
  })

  it("lays the project's release graph before the session starts, and says so", async () => {
    openTerminal.mockResolvedValue(terminal())
    releaseOnRun.mockResolvedValue('Release')

    await mountButton().get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(releaseOnRun).toHaveBeenCalledWith(TASK)
    expect(releaseOnRun.mock.invocationCallOrder[0]!).toBeLessThan(openTerminal.mock.invocationCallOrder[0]!)
    expect(useToastStore().toasts.map((toast) => toast.message)).toContain('Release laid on the task before it runs.')
  })

  it('starts the session anyway when the release graph cannot be laid, saying why', async () => {
    openTerminal.mockResolvedValue(terminal())
    releaseOnRun.mockRejectedValue(new Error('This task has 1 draft steps.'))

    await mountButton().get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalled()
    expect(useToastStore().toasts.map((toast) => toast.message)).toContain('This task has 1 draft steps.')
  })

  it('asks for the project folder instead of launching when there is none', async () => {
    const button = mountButton(null).get('[data-testid="open-terminal"]')

    await button.trigger('click')

    expect(openTerminal).not.toHaveBeenCalled()
    expect(button.attributes('data-phase')).toBe('idle')
  })

  it('says why the shown provider cannot start, links its setup and takes no menu action', async () => {
    installBothSessionProviders()
    setPreferredProvider('CODEX')
    useCodexStore().setup = { status: 'CLI_MISSING' }
    const wrapper = mountButton()

    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
    const menu = document.body.querySelector('[data-testid="run-menu"]')!
    menuButton('discuss').click()
    await flushPromises()

    expect(menu.textContent).toContain('Codex CLI is not installed.')
    expect(wrapper.findComponent(RouterLinkStub).props('to')).toBe('/settings/sessions')
    expect(menuButton('discuss').getAttribute('aria-disabled')).toBe('true')
    expect(openTerminal).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('dims a provider whose CLI is missing, saying why, and cannot be switched to it', async () => {
    installBothSessionProviders()
    useCodexStore().setup = { status: 'CLI_MISSING' }
    const wrapper = mountButton()
    const codex = wrapper.get('[data-testid="provider-switch-CODEX"]')

    await codex.trigger('click')

    expect(codex.attributes('aria-disabled')).toBe('true')
    expect(codex.attributes('title')).toBe('Codex CLI is not installed.')
    expect(codex.attributes('aria-checked')).toBe('false')
    wrapper.unmount()
  })

  it('holds the switch on the provider of a live session and says to close it', async () => {
    setPreferredProvider('CODEX')
    useTerminalStore().terminals = [{ ...terminal(), kind: 'CHAT', mode: 'DISCUSS' }]
    const wrapper = mountButton()
    const codex = wrapper.get('[data-testid="provider-switch-CODEX"]')

    await codex.trigger('click')

    expect(wrapper.get('[data-testid="provider-switch-CLAUDE"]').attributes('aria-checked')).toBe('true')
    expect(codex.attributes('aria-disabled')).toBe('true')
    expect(codex.attributes('title')).toBe('Claude Code Chat is open on this task. Close it to use Codex.')
    wrapper.unmount()
  })

  it('warns under Run in queue that the live session has to be closed first', async () => {
    useTerminalStore().terminals = [terminal()]
    const wrapper = mountButton()

    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')

    expect(menuButton('run-in-queue').textContent).toContain('Close it before the queue starts this task.')
    wrapper.unmount()
  })

  it('splits the ways to run from the ways to prepare with one separator', async () => {
    installBothSessionProviders()
    const wrapper = mountButton()

    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')

    const rows = [...document.body.querySelector('[data-testid="run-menu"]')!.children].map((row) =>
      row.getAttribute('role') === 'separator' ? '---' : row.textContent?.trim()
    )
    expect(rows).toEqual(['With Claude Code', 'Run in queue', '---', 'Discuss', 'Plan here', 'Plan a flow'])
    wrapper.unmount()
  })

  it('walks the menu with the arrow keys straight over the separator', async () => {
    const wrapper = mount(OpenTerminalButton, { props: { taskId: TASK, folder: '/code/vega' }, attachTo: document.body })
    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
    const menu = document.body.querySelector<HTMLElement>('[data-testid="run-menu"]')!
    const press = (key: string): string | undefined => {
      menu.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
      return document.activeElement?.textContent?.trim()
    }

    menuButton('run-in-queue').focus()
    expect(press('ArrowDown')).toBe('Discuss')
    expect(press('ArrowUp')).toBe('Run in queue')
    expect(press('ArrowUp')).toBe('Plan a flow')
    wrapper.unmount()
  })

  it('moves the switch and its focus with the arrow keys', async () => {
    const wrapper = mountButton()
    const group = wrapper.get('[data-testid="provider-switch"]')

    await group.trigger('keydown', { key: 'ArrowRight' })
    await flushPromises()

    const codex = wrapper.get('[data-testid="provider-switch-CODEX"]')
    expect(codex.attributes('aria-checked')).toBe('true')
    expect(codex.attributes('tabindex')).toBe('0')
    expect(document.activeElement).toBe(codex.element)
    wrapper.unmount()
  })

  it.each([
    { defaultProvider: 'CODEX', provider: 'CLAUDE', model: 'sonnet', effort: 'low', kind: 'CHAT', skipPermissions: true },
    { defaultProvider: 'CLAUDE', provider: 'CODEX', model: 'gpt-test', effort: 'xhigh', kind: 'CHAT', skipPermissions: false }
  ] as const)('runs with $provider once the switch shows it', async ({ defaultProvider, ...choices }) => {
    installBothSessionProviders()
    setPreferredProvider(defaultProvider)
    setPreferredModel('sonnet')
    setPreferredEffort('low')
    setPreferredSessionKind('chat')
    setSkipsPermissions(true)
    setCodexModel('gpt-test')
    setCodexEffort('xhigh')
    openTerminal.mockResolvedValue({ ...terminal(), provider: choices.provider, kind: choices.kind })
    const wrapper = mountButton()

    await switchTo(wrapper, choices.provider)
    await wrapper.get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining(choices))
    expect(preferredProvider()).toBe(defaultProvider)
    wrapper.unmount()
  })

  it('keeps the switch still while a launch is under way', async () => {
    openTerminal.mockReturnValue(new Promise<Terminal>(() => undefined))
    const wrapper = mountButton()
    await wrapper.get('[data-testid="open-terminal"]').trigger('click')

    await switchTo(wrapper, 'CODEX')

    expect(wrapper.get('[data-testid="provider-switch-CLAUDE"]').attributes('aria-checked')).toBe('true')
    wrapper.unmount()
  })

  it.each(['CLAUDE', 'CODEX'] as const)('queues once with %s and identifies the provider in feedback', async (provider) => {
    runTaskInQueue.mockResolvedValue(queueWith([{ ...queueItem(TASK, 0, 'RUNNING'), provider }]))
    const toast = useToastStore()
    const wrapper = mountButton()

    await chooseRunInQueue(wrapper, provider)

    expect(runTaskInQueue).toHaveBeenCalledTimes(1)
    expect(runTaskInQueue).toHaveBeenCalledWith(TASK, provider)
    expect(openTerminal).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toBe(`Running with ${provider === 'CODEX' ? 'Codex' : 'Claude Code'}`)
    wrapper.unmount()
  })

  it('says how many places wait ahead of a task that has to wait its turn', async () => {
    const other = 't-2' as TaskId
    runTaskInQueue.mockResolvedValue(
      queueWith([queueItem(other, 0, 'RUNNING'), queueItem(other, 1, 'QUEUED'), queueItem(TASK, 2, 'QUEUED')])
    )
    const toast = useToastStore()
    const wrapper = mountButton()

    await chooseRunInQueue(wrapper)

    expect(toast.toasts[0]?.message).toBe('Queued with Claude Code · 2 ahead')
    wrapper.unmount()
  })

  it('shows the refusal as a toast when the task is already queued', async () => {
    runTaskInQueue.mockRejectedValue(new Error('Already in the queue'))
    const toast = useToastStore()
    const wrapper = mountButton()

    await chooseRunInQueue(wrapper)

    expect(toast.toasts[0]?.message).toContain('Already in the queue')
    wrapper.unmount()
  })

  it('asks for the project folder and does not queue when there is none', async () => {
    const toast = useToastStore()
    const wrapper = mountButton(null)

    await chooseRunInQueue(wrapper)

    expect(runTaskInQueue).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toContain('folder')
    wrapper.unmount()
  })

  it('says nothing will run and does not queue when only drafts are left', async () => {
    const console_ = useConsoleStore()
    console_.tasks = [checklistTask()]
    console_.steps = [step('CLAIMED'), { ...step('DRAFT'), id: 's-2' as TaskStepId }]
    const toast = useToastStore()
    const wrapper = mountButton()

    await chooseRunInQueue(wrapper)

    expect(runTaskInQueue).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toContain('Nothing to run')
    wrapper.unmount()
  })

  it('does nothing from the arrow while the key is busy', async () => {
    openTerminal.mockReturnValue(new Promise<Terminal>(() => undefined))
    const wrapper = mountButton()
    await wrapper.get('[data-testid="open-terminal"]').trigger('click')

    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')

    expect(document.body.querySelector('[data-testid="run-menu"]')).toBeNull()
    wrapper.unmount()
  })
})

describe('planning from the Run here arrow', () => {
  function stepOf(id: string, state: TaskStep['state']): TaskStep {
    return { id: id as TaskStepId, taskId: TASK, state } as TaskStep
  }

  function mountPlanning(folder: string | null = '/code/vega') {
    return mount(OpenTerminalButton, {
      props: { taskId: TASK, folder },
      global: { stubs: { AppConfirm: true, RouterLink: RouterLinkStub }, renderStubDefaultSlot: true },
      attachTo: document.body
    })
  }

  async function choose(wrapper: ReturnType<typeof mountPlanning>, testid: string): Promise<void> {
    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
    menuButton(testid).click()
    await flushPromises()
  }

  function liveTerminal(): Terminal {
    return { ...terminal(), live: true }
  }

  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
    openTerminal.mockReset()
    sendTerminalInput.mockReset()
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('opens a planning session when none is live on the task, and stays on the view it is on', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), id: 'term-2' as TerminalId })
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')

    await choose(mountPlanning(), 'plan-here')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN' }))
    expect(openPane).not.toHaveBeenCalled()
  })

  it('routes plan to the server when the task already has a session', async () => {
    const existing = liveTerminal()
    useTerminalStore().terminals = [existing]
    openTerminal.mockResolvedValue(existing)

    await choose(mountPlanning(null), 'plan-here')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN', provider: 'CLAUDE' }))
    expect(sendTerminalInput).not.toHaveBeenCalled()
  })

  it('asks for the project folder instead of planning when there is neither a folder nor a session', async () => {
    const toast = useToastStore()

    await choose(mountPlanning(null), 'plan-here')

    expect(openTerminal).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toContain('folder')
  })

  it('offers the flow only while the task has no steps, and the replan only while a step can be replaced', async () => {
    const console_ = useConsoleStore()
    const offered = async (steps: TaskStep[]): Promise<string[]> => {
      console_.steps = steps
      const wrapper = mountPlanning()
      await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
      const labels = menuLabels()
      wrapper.unmount()
      document.body.innerHTML = ''
      return labels
    }

    expect(await offered([])).toEqual(['Run in queue', 'Discuss', 'Plan here', 'Plan a flow'])
    expect(await offered([stepOf('a', 'DONE'), stepOf('b', 'RUNNING')])).toEqual(['Run in queue', 'Discuss', 'Plan here'])
    expect(await offered([stepOf('a', 'DONE'), stepOf('b', 'OPEN')])).toEqual(['Run in queue', 'Discuss', 'Plan here', 'Replan'])
    expect(await offered([stepOf('a', 'DRAFT')])).toEqual(['Run in queue', 'Discuss', 'Plan here', 'Replan'])
  })

  it('opens a discussion when none is live on the task, says so, and stays on the view it is on', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), id: 'term-2' as TerminalId })
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')

    await choose(mountPlanning(), 'discuss')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'DISCUSS' }))
    expect(openPane).not.toHaveBeenCalled()
    expect(useToastStore().toasts.map((toast) => toast.message)).toContain('Discussing. What you agree lands on the task.')
  })

  it('routes discuss to the server when the task already has a session', async () => {
    const existing = liveTerminal()
    useTerminalStore().terminals = [existing]
    openTerminal.mockResolvedValue(existing)

    await choose(mountPlanning(null), 'discuss')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'DISCUSS', provider: 'CLAUDE' }))
    expect(sendTerminalInput).not.toHaveBeenCalled()
  })

  it('opens a session in flow mode when Plan a flow is chosen', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), id: 'term-2' as TerminalId })

    await choose(mountPlanning(), 'plan-flow')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN_FLOW' }))
  })

  it.each([
    ['discuss', 'DISCUSS'],
    ['plan-here', 'PLAN'],
    ['plan-flow', 'PLAN_FLOW']
  ] as const)('dispatches %s once with the provider the switch shows', async (testid, mode) => {
    setPreferredProvider('CLAUDE')
    openTerminal.mockResolvedValue({ ...terminal(), provider: 'CODEX' })
    const wrapper = mountPlanning()
    await wrapper.get('[data-testid="provider-switch-CODEX"]').trigger('click')

    await choose(wrapper, testid)

    expect(openTerminal).toHaveBeenCalledTimes(1)
    expect(openTerminal).toHaveBeenCalledWith(
      { taskId: TASK },
      expect.objectContaining({ mode, provider: 'CODEX' })
    )
    expect(preferredProvider()).toBe('CLAUDE')
  })

  describe('replanning', () => {
    function withReplaceableSteps(count: number): void {
      useConsoleStore().steps = Array.from({ length: count }, (_, index) => stepOf(`s-${index}`, 'OPEN'))
    }

    it('asks before anything is deleted, saying how many steps go', async () => {
      withReplaceableSteps(3)
      const wrapper = mountPlanning()

      await choose(wrapper, 'replan')

      expect(wrapper.findComponent(AppConfirm).attributes('blast')).toContain('3 steps deleted')
      expect(openTerminal).not.toHaveBeenCalled()
      expect(sendTerminalInput).not.toHaveBeenCalled()
    })

    it('replans with the provider the switch shows', async () => {
      withReplaceableSteps(1)
      openTerminal.mockResolvedValue({ ...terminal(), provider: 'CODEX' })
      const wrapper = mountPlanning()
      await wrapper.get('[data-testid="provider-switch-CODEX"]').trigger('click')

      await choose(wrapper, 'replan')
      await wrapper.findComponent(AppConfirm).vm.$emit('confirm')
      await flushPromises()

      expect(openTerminal).toHaveBeenCalledWith(
        { taskId: TASK },
        expect.objectContaining({ mode: 'REPLAN', provider: 'CODEX' })
      )
    })

    it('says one step in the singular', async () => {
      withReplaceableSteps(1)
      const wrapper = mountPlanning()

      await choose(wrapper, 'replan')

      expect(wrapper.findComponent(AppConfirm).attributes('blast')).toContain('1 step deleted')
    })

    it('opens a replan session once confirmed, with what the plan missed', async () => {
      withReplaceableSteps(2)
      openTerminal.mockResolvedValue({ ...terminal(), id: 'term-2' as TerminalId })
      const wrapper = mountPlanning()

      await choose(wrapper, 'replan')
      await wrapper.get('[data-testid="replan-message"]').setValue('also cover the csv export')
      await wrapper.findComponent(AppConfirm).vm.$emit('confirm')
      await flushPromises()

      expect(openTerminal).toHaveBeenCalledWith(
        { taskId: TASK },
        expect.objectContaining({ mode: 'REPLAN', request: 'also cover the csv export' })
      )
      expect(wrapper.findComponent(AppConfirm).exists()).toBe(false)
    })

    it('does nothing when the confirmation is dismissed', async () => {
      withReplaceableSteps(2)
      const wrapper = mountPlanning()

      await choose(wrapper, 'replan')
      await wrapper.findComponent(AppConfirm).vm.$emit('cancel')
      await flushPromises()

      expect(wrapper.findComponent(AppConfirm).exists()).toBe(false)
      expect(openTerminal).not.toHaveBeenCalled()
      expect(sendTerminalInput).not.toHaveBeenCalled()
    })
  })
})
