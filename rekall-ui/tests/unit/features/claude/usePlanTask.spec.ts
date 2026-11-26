import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { effectScope } from 'vue'
import { flushPromises } from '@vue/test-utils'
import { usePlanTask, type PlanTask } from '@/features/claude/composables/usePlanTask'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'
import { CHAT_ID, chatSession } from '../chat/support'
import { installStorage } from '../../support/memoryStorage'

const openTerminal = vi.fn()
const push = vi.fn()

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...a: unknown[]) => openTerminal(...a),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({ currentRoute: { value: { name: 'task' } }, push })
}))

const TASK = 't-1' as TaskId

function planning(): PlanTask {
  return effectScope().run(() => usePlanTask(() => TASK, () => '/code/vega', 'Set the folder'))!
}

function shown(): unknown[] {
  return [useConsoleStore().selectedTaskId, useConsoleStore().paneFocus, useChatStore().activeChatId]
}

describe('planning a task', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    openTerminal.mockReset()
    push.mockReset()
  })

  it.each(['PLAN', 'PLAN_FLOW', 'DISCUSS', 'REFINE'] as const)('shows the chat a %s opens full-size', async (mode) => {
    openTerminal.mockResolvedValue(chatSession({ mode }))

    planning().start(mode)
    await flushPromises()

    expect(shown()).toEqual([TASK, 'chat', CHAT_ID])
    expect(push).toHaveBeenCalledWith({ name: 'console' })
    expect(useTerminalStore().floatingId).toBeNull()
  })

  it('shows the chat full-size once a replan is confirmed, not before', async () => {
    openTerminal.mockResolvedValue(chatSession({ mode: 'REPLAN' }))
    const plan = planning()

    plan.start('REPLAN')
    await flushPromises()
    expect(push).not.toHaveBeenCalled()
    await plan.confirmReplan()

    expect(shown()).toEqual([TASK, 'chat', CHAT_ID])
  })

  it('floats the terminal it opens and stays on the page', async () => {
    openTerminal.mockResolvedValue(chatSession({ kind: 'TERMINAL' }))

    planning().start('PLAN')
    await flushPromises()

    expect(useTerminalStore().floatingId).toBe(CHAT_ID)
    expect(push).not.toHaveBeenCalled()
  })
})
