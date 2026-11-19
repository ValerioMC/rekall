import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskComposer from '@/features/catalog/components/TaskComposer.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { QUICK_TASK_HELP } from '@/features/catalog/lib/quick-task'
import { preferredProvider, setCodexEffort, setCodexModel } from '@/common/config/session-launch'
import { installStorage } from '../../support/memoryStorage'
import { installBothSessionProviders } from '../../support/sessionProviders'
import { taskFixture } from '../../support/taskFixture'
import type { QuickTaskStarted } from '@/features/catalog/api/quick-task.api'
import type { Project } from '@/features/catalog/model/catalog'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId } from '@/model/branded'

const openSession = vi.fn()

vi.mock('@/features/claude/composables/useOpenSession', () => ({ useOpenSession: () => openSession }))

const PROJECT = { id: 'p1' as ProjectId, title: 'Vega', repoFolder: '/code/vega' } as Project
const OTHER = { id: 'p2' as ProjectId, title: 'Orion', repoFolder: '/code/orion' } as Project
const TERMINAL = { id: 'term-1', taskId: 't-quick', kind: 'TERMINAL' } as unknown as Terminal
const STARTED: QuickTaskStarted = { task: { ...taskFixture('t-quick' as TaskId), quick: true }, terminal: TERMINAL }

function mountComposer(project: Project = PROJECT, choices: readonly Project[] = []) {
  return mount(TaskComposer, { attachTo: document.body, props: { project, choices } })
}

async function type(wrapper: ReturnType<typeof mountComposer>, text: string): Promise<void> {
  await wrapper.get('[data-testid="quick-task-input"]').setValue(text)
}

const field = (wrapper: ReturnType<typeof mountComposer>) => wrapper.get('[data-testid="quick-task-input"]').element as HTMLTextAreaElement

describe('TaskComposer', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
    openSession.mockReset()
    document.body.replaceChildren()
  })

  it('explains what a quick task does and keeps the start key off while the field is empty', async () => {
    const wrapper = mountComposer()
    await wrapper.get('[data-testid="quick-task-input"]').trigger('focus')

    expect(wrapper.get('[data-testid="quick-task-help"]').text()).toBe(QUICK_TASK_HELP)
    expect(wrapper.get('[data-testid="quick-task-start"]').attributes('disabled')).toBeDefined()
  })

  it('starts the text on Enter, shows the session it opened and clears the field', async () => {
    const store = useConsoleStore()
    const create = vi.spyOn(store, 'createQuickTask').mockResolvedValue(STARTED)
    const adopt = vi.spyOn(useTerminalStore(), 'adoptOpened')
    const wrapper = mountComposer()

    await type(wrapper, 'Fix the typo in README')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')
    await flushPromises()

    expect(create).toHaveBeenCalledWith(PROJECT.id, expect.objectContaining({ text: 'Fix the typo in README' }))
    expect(adopt).toHaveBeenCalledWith(TERMINAL)
    expect(openSession).toHaveBeenCalledWith(TERMINAL)
    expect(field(wrapper).value).toBe('')
  })

  it('goes to a new line on Shift+Enter instead of starting', async () => {
    const create = vi.spyOn(useConsoleStore(), 'createQuickTask').mockResolvedValue(STARTED)
    const wrapper = mountComposer()

    await type(wrapper, 'First line')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter', { shiftKey: true })

    expect(create).not.toHaveBeenCalled()
  })

  it('starts a quick task with Codex from the arrow menu', async () => {
    installBothSessionProviders()
    setCodexModel('gpt-test')
    setCodexEffort('high')
    const create = vi.spyOn(useConsoleStore(), 'createQuickTask').mockResolvedValue(STARTED)
    const wrapper = mountComposer()

    await type(wrapper, 'Fix the typo')
    await wrapper.get('[data-testid="split-button-arrow"]').trigger('click')
    expect(document.body.querySelector('[data-testid="split-button-option-CODEX"] svg')?.getAttribute('data-provider')).toBe('CODEX')
    expect(document.body.querySelector('[data-testid="split-button-option-CLAUDE"] svg')?.getAttribute('data-provider')).toBe('CLAUDE')
    document.body.querySelector<HTMLButtonElement>('[data-testid="split-button-option-CODEX"]')!.click()
    await flushPromises()

    expect(create).toHaveBeenCalledWith(PROJECT.id, expect.objectContaining({
      text: 'Fix the typo', provider: 'CODEX', model: 'gpt-test', effort: 'high'
    }))
    expect(preferredProvider()).toBe('CLAUDE')
    expect(field(wrapper).value).toBe('')
    wrapper.unmount()
  })

  it('starts nothing from blank text', async () => {
    const create = vi.spyOn(useConsoleStore(), 'createQuickTask').mockResolvedValue(STARTED)
    const wrapper = mountComposer()

    await type(wrapper, '   ')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')

    expect(create).not.toHaveBeenCalled()
  })

  it('says why it cannot start on a project with no folder, and still offers to plan', async () => {
    const wrapper = mountComposer({ ...PROJECT, repoFolder: null } as Project)
    await type(wrapper, 'Draft it')

    expect(wrapper.get('[data-testid="quick-task-help"]').text()).toContain("Set this project's folder")
    expect(wrapper.get('[data-testid="quick-task-start"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="split-button-arrow"]').attributes('disabled')).toBeUndefined()
  })

  it('keeps the text and tells the reader when the server refuses', async () => {
    vi.spyOn(useConsoleStore(), 'createQuickTask').mockRejectedValue(new Error('The session could not open.'))
    const notify = vi.spyOn(useToastStore(), 'notifyError')
    const wrapper = mountComposer()

    await type(wrapper, 'Fix it')
    await wrapper.get('[data-testid="quick-task-input"]').trigger('keydown.enter')
    await flushPromises()

    expect(notify).toHaveBeenCalled()
    expect(openSession).not.toHaveBeenCalled()
    expect(field(wrapper).value).toBe('Fix it')
  })

  it('offers the plan dialog with the typed text as its title', async () => {
    const wrapper = mountComposer()
    await type(wrapper, '  Plan the export  ')

    await wrapper.get('[data-testid="split-button-arrow"]').trigger('click')
    document.body.querySelector<HTMLElement>('[data-testid="split-button-option-plan"]')!.click()

    expect(wrapper.emitted('plan')).toEqual([['Plan the export']])
  })

  it('shows the project folder when one project is in view and a picker when several are offered', async () => {
    const fixed = mountComposer()
    expect(fixed.get('[data-testid="composer-folder"]').text()).toBe('/code/vega')
    expect(fixed.find('[data-testid="composer-project"]').exists()).toBe(false)

    const picking = mountComposer(PROJECT, [PROJECT, OTHER])
    await picking.get('[data-testid="composer-project"]').trigger('click')
    await picking.findAll('[data-testid="popover-menu-item"]').find((item) => item.text() === 'Orion')!.trigger('click')

    expect(picking.get('[data-testid="composer-project"]').text()).toBe('Vega')
    expect(picking.emitted('choose')).toEqual([['p2']])
  })
})
