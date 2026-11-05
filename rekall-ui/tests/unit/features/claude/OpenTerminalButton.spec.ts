import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
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

const runTaskInQueue = vi.fn()

vi.mock('@/features/queue/api/runQueue.api', () => ({
  runTaskInQueue: (...args: unknown[]) => runTaskInQueue(...args)
}))

const TASK = 't-1' as TaskId
const STEP = 's-1' as TaskStepId
const ANCHOR = 'project:vega task:report-builder'

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
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
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
    taskId,
    taskTitle: 'Task',
    taskLabel: 'task',
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

async function chooseRunInQueue(wrapper: ReturnType<typeof mountButton>): Promise<void> {
  await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
  document.body.querySelector<HTMLElement>('[data-testid="run-in-queue"]')?.click()
  await flushPromises()
}

function menuLabels(): string[] {
  return [...document.body.querySelectorAll('[role="menuitem"]')].map((item) => item.textContent?.trim() ?? '')
}

function mountButton(folder: string | null = '/code/vega', stepId: TaskStepId | null = null) {
  useConsoleStore().releaseOnRun = releaseOnRun
  return mount(OpenTerminalButton, { props: { taskId: TASK, anchor: ANCHOR, folder, stepId } })
}

describe('the Run here button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    openTerminal.mockReset()
    releaseOnRun.mockReset().mockResolvedValue(null)
    runTaskInQueue.mockReset()
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

  it('opens a menu from the arrow with Run in queue and Plan here, each a name with no description', async () => {
    const wrapper = mountButton()

    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')

    const menu = document.body.querySelector('[data-testid="run-menu"]')
    expect(menuLabels()).toEqual(['Run in queue', 'Plan here', 'Plan a flow'])
    expect(menu?.querySelectorAll('[role="menuitem"] svg')).toHaveLength(3)
    expect(wrapper.get('[data-testid="run-menu-arrow"]').attributes('aria-haspopup')).toBe('menu')
    wrapper.unmount()
  })

  it('queues the task without opening a terminal and says it runs now', async () => {
    runTaskInQueue.mockResolvedValue(queueWith([queueItem(TASK, 0, 'RUNNING')]))
    const toast = useToastStore()
    const wrapper = mountButton()

    await chooseRunInQueue(wrapper)

    expect(runTaskInQueue).toHaveBeenCalledWith(TASK)
    expect(openTerminal).not.toHaveBeenCalled()
    expect(toast.toasts[0]?.message).toBe('Running now in the queue')
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

    expect(toast.toasts[0]?.message).toBe('Queued, 2 ahead')
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
      props: { taskId: TASK, anchor: ANCHOR, folder },
      global: { stubs: { AppConfirm: true }, renderStubDefaultSlot: true },
      attachTo: document.body
    })
  }

  async function choose(wrapper: ReturnType<typeof mountPlanning>, testid: string): Promise<void> {
    await wrapper.get('[data-testid="run-menu-arrow"]').trigger('click')
    document.body.querySelector<HTMLElement>(`[data-testid="${testid}"]`)?.click()
    await flushPromises()
  }

  function liveTerminal(): Terminal {
    return { ...terminal(), live: true }
  }

  beforeEach(() => {
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

  it('types the plan line into the session live on the task', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    useTerminalStore().terminals = [liveTerminal()]

    await choose(mountPlanning(null), 'plan-here')

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} plan\r`)
    expect(openTerminal).not.toHaveBeenCalled()
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

    expect(await offered([])).toEqual(['Run in queue', 'Plan here', 'Plan a flow'])
    expect(await offered([stepOf('a', 'DONE'), stepOf('b', 'RUNNING')])).toEqual(['Run in queue', 'Plan here'])
    expect(await offered([stepOf('a', 'DONE'), stepOf('b', 'OPEN')])).toEqual(['Run in queue', 'Plan here', 'Replan'])
    expect(await offered([stepOf('a', 'DRAFT')])).toEqual(['Run in queue', 'Plan here', 'Replan'])
  })

  it('opens a session in flow mode when Plan a flow is chosen', async () => {
    openTerminal.mockResolvedValue({ ...terminal(), id: 'term-2' as TerminalId })

    await choose(mountPlanning(), 'plan-flow')

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'PLAN_FLOW' }))
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
