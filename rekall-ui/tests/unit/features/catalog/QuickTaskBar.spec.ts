import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import QuickTaskBar from '@/features/catalog/components/QuickTaskBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { QUICK_TASK_BUSY_REASON, QUICK_TASK_HELP } from '@/features/catalog/lib/quick-task'
import { taskFixture } from '../../support/taskFixture'
import type { QuickTaskStarted } from '@/features/catalog/api/quick-task.api'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId } from '@/model/branded'

const openSession = vi.fn()

vi.mock('@/features/claude/composables/useOpenSession', () => ({ useOpenSession: () => openSession }))

const PROJECT = { id: 'p1' as ProjectId, repoFolder: '/code/vega' } as Project
const TERMINAL = { id: 'term-1', taskId: 't-quick', kind: 'TERMINAL' } as unknown as Terminal
const STARTED: QuickTaskStarted = { task: { ...taskFixture('t-quick' as TaskId), quick: true }, terminal: TERMINAL }

function mountBar(project: Project = PROJECT) {
  return mount(QuickTaskBar, { props: { project } })
}

async function type(wrapper: ReturnType<typeof mountBar>, text: string): Promise<void> {
  await wrapper.get('[data-testid="quick-task-input"]').setValue(text)
}

describe('QuickTaskBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    openSession.mockReset()
  })

  it('explains what a quick task does and keeps the start key off while the field is empty', () => {
    const wrapper = mountBar()
    expect(wrapper.get('[data-testid="quick-task-help"]').text()).toBe(QUICK_TASK_HELP)
    expect(wrapper.get('[data-testid="quick-task-start"]').attributes('disabled')).toBeDefined()
  })

  it('starts the text on Enter, shows the session it opened and clears the field', async () => {
    const store = useConsoleStore()
    const create = vi.spyOn(store, 'createQuickTask').mockResolvedValue(STARTED)
    const adopt = vi.spyOn(useTerminalStore(), 'adoptOpened')
    const wrapper = mountBar()

    await type(wrapper, 'Fix the typo in README')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')
    await flushPromises()

    expect(create).toHaveBeenCalledWith(PROJECT.id, expect.objectContaining({ text: 'Fix the typo in README' }))
    expect(adopt).toHaveBeenCalledWith(TERMINAL)
    expect(openSession).toHaveBeenCalledWith(TERMINAL)
    expect((wrapper.get('[data-testid="quick-task-input"]').element as HTMLTextAreaElement).value).toBe('')
  })

  it('goes to a new line on Shift+Enter instead of starting', async () => {
    const create = vi.spyOn(useConsoleStore(), 'createQuickTask').mockResolvedValue(STARTED)
    const wrapper = mountBar()

    await type(wrapper, 'First line')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter', { shiftKey: true })

    expect(create).not.toHaveBeenCalled()
  })

  it('starts nothing from blank text', async () => {
    const create = vi.spyOn(useConsoleStore(), 'createQuickTask').mockResolvedValue(STARTED)
    const wrapper = mountBar()

    await type(wrapper, '   ')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')

    expect(create).not.toHaveBeenCalled()
  })

  it('is held back with the reason while a quick task is still being worked on the project', async () => {
    const store = useConsoleStore()
    store.tasks = [{ ...STARTED.task, reviewState: 'RUNNING' } as Task]
    const wrapper = mountBar()

    expect(wrapper.get('[data-testid="quick-task-help"]').text()).toBe(QUICK_TASK_BUSY_REASON)
    expect(wrapper.get('[data-testid="quick-task-input"]').attributes('disabled')).toBeDefined()
  })

  it('is free again once the wrapup claimed the earlier quick task', () => {
    const store = useConsoleStore()
    store.tasks = [{ ...STARTED.task, reviewState: 'CLAIMED' } as Task]

    expect(mountBar().get('[data-testid="quick-task-input"]').attributes('disabled')).toBeUndefined()
  })

  it('is free again when the earlier quick task lost its session and is back to open', () => {
    const store = useConsoleStore()
    store.tasks = [{ ...STARTED.task, reviewState: 'OPEN' } as Task]

    expect(mountBar().get('[data-testid="quick-task-input"]').attributes('disabled')).toBeUndefined()
  })

  it('says why it cannot start on a project with no folder', () => {
    const wrapper = mountBar({ ...PROJECT, repoFolder: null } as Project)

    expect(wrapper.get('[data-testid="quick-task-help"]').text()).toContain("Set this project's folder")
    expect(wrapper.get('[data-testid="quick-task-input"]').attributes('disabled')).toBeDefined()
  })

  it('keeps the text and tells the reader when the server refuses', async () => {
    const store = useConsoleStore()
    vi.spyOn(store, 'createQuickTask').mockRejectedValue(new Error('This project already has a quick task being worked.'))
    const notify = vi.spyOn(useToastStore(), 'notifyError')
    const wrapper = mountBar()

    await type(wrapper, 'Fix it')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')
    await flushPromises()

    expect(notify).toHaveBeenCalled()
    expect(openSession).not.toHaveBeenCalled()
    expect((wrapper.get('[data-testid="quick-task-input"]').element as HTMLTextAreaElement).value).toBe('Fix it')
  })
})
