import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import OpenTerminalButton from '@/components/claude/OpenTerminalButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Task, TaskStep } from '@/model/catalog'
import type { Terminal } from '@/model/terminal'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

const openTerminal = vi.fn()

vi.mock('@/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...args: unknown[]) => openTerminal(...args),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
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
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    skipPermissions: true,
    model: null,
    effort: null,
    live: true,
    startedAt: '2026-09-22T10:00:00Z',
    lastActivityAt: '2026-09-22T10:00:00Z'
  }
}

function mountButton(folder: string | null = '/code/vega', stepId: TaskStepId | null = null) {
  return mount(OpenTerminalButton, { props: { taskId: TASK, folder, stepId } })
}

describe('the Run here button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    openTerminal.mockReset()
    vi.useFakeTimers()
  })

  afterEach(() => vi.useRealTimers())

  it('starts the session without switching the console to its terminal', async () => {
    openTerminal.mockResolvedValue(terminal())
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')
    const wrapper = mountButton()

    await wrapper.get('[data-testid="open-terminal"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith(TASK, expect.objectContaining({ stepId: null }))
    expect(openPane).not.toHaveBeenCalled()
  })

  it('runs the play mark out, then closes the ring on a check, then shows the work in progress', async () => {
    openTerminal.mockResolvedValue(terminal())
    const wrapper = mountButton()
    const button = wrapper.get('[data-testid="open-terminal"]')

    await button.trigger('click')
    expect(button.attributes('data-phase')).toBe('launching')

    await vi.advanceTimersByTimeAsync(380)
    expect(button.attributes('data-phase')).toBe('done')
    expect(wrapper.find('.run-check').exists()).toBe(true)

    await vi.advanceTimersByTimeAsync(1200)
    expect(button.attributes('data-phase')).toBe('working')
    expect(wrapper.find('.run-orbit').exists()).toBe(true)
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

  it('asks for the project folder instead of launching when there is none', async () => {
    const button = mountButton(null).get('[data-testid="open-terminal"]')

    await button.trigger('click')

    expect(openTerminal).not.toHaveBeenCalled()
    expect(button.attributes('data-phase')).toBe('idle')
  })
})
