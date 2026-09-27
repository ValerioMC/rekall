import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import OpenTerminalButton from '@/components/claude/OpenTerminalButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
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

function mountButton(folder: string | null = '/code/vega') {
  return mount(OpenTerminalButton, { props: { taskId: TASK, folder } })
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

  it('runs the play mark out, then closes the ring on a check, then returns to rest', async () => {
    openTerminal.mockResolvedValue(terminal())
    const wrapper = mountButton()
    const button = wrapper.get('[data-testid="open-terminal"]')

    await button.trigger('click')
    expect(button.attributes('data-phase')).toBe('launching')

    await vi.advanceTimersByTimeAsync(380)
    expect(button.attributes('data-phase')).toBe('done')
    expect(wrapper.find('.run-check').exists()).toBe(true)

    await vi.advanceTimersByTimeAsync(2200)
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
