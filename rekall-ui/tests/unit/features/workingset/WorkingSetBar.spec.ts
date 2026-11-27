import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import * as api from '@/features/workingset/api/working-set.api'
import WorkingSetBar from '@/features/workingset/components/WorkingSetBar.vue'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TerminalId } from '@/model/branded'
import { step } from '../project/project-task-fixtures'
import { company, project, task } from './fixtures'

vi.mock('@/features/workingset/api/working-set.api')

const T1 = 't1' as TaskId
const T2 = 't2' as TaskId
const T3 = 't3' as TaskId
const VEGA = 'vega' as ProjectId
const ATLAS = 'atlas' as ProjectId

const Page = { render: () => null }

function showTerminalOf(taskId: TaskId): void {
  const catalog = useConsoleStore()
  catalog.selectTask(taskId)
  catalog.openTerminal()
}

function markedTasks(bar: ReturnType<typeof mount>): (string | undefined)[] {
  return bar.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('aria-selected'))
}

function liveTerminalOn(taskId: TaskId): void {
  useTerminalStore().terminals = [{ id: `term-${taskId}` as TerminalId, taskId, kind: 'TERMINAL', live: true } as Terminal]
}

function liveChatOn(taskId: TaskId): void {
  useTerminalStore().terminals = [{ id: `chat-${taskId}` as TerminalId, taskId, kind: 'CHAT', live: true } as Terminal]
}

function showChatOf(taskId: TaskId): void {
  const catalog = useConsoleStore()
  catalog.selectTask(taskId)
  catalog.openChat()
  useChatStore().select(`chat-${taskId}` as TerminalId)
}

async function barAt(path: string): Promise<{ bar: ReturnType<typeof mount>; router: Router }> {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/task/:id/:tab?/:noteId?', name: 'task', component: Page },
      { path: '/projects/:id/:tab?', name: 'project-detail', component: Page },
      { path: '/console', name: 'console', component: Page },
      { path: '/settings', name: 'settings', component: Page }
    ]
  })
  await router.push(path)
  await router.isReady()
  const bar = mount(WorkingSetBar, { global: { plugins: [router] } })
  await flushPromises()
  return { bar, router }
}

describe('the working-set bar', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    const catalog = useConsoleStore()
    catalog.companies = [company('c1', 'Northwind')]
    catalog.projects = [project('vega', 'Vega', 'c1'), project('atlas', 'Atlas', 'c1')]
    catalog.tasks = [task('t1', 'vega'), task('t2', 'vega'), task('t3', 'atlas')]
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({
      tasks: [{ taskId: T1, projectId: VEGA }, { taskId: T2, projectId: VEGA }, { taskId: T3, projectId: ATLAS }],
      places: [{ projectId: ATLAS, taskId: T3 }]
    })
    await useWorkingSetStore().load()
  })

  it('shows both rows on a task page, marking its project and task', async () => {
    const { bar } = await barAt('/task/t2/description')

    expect(bar.findAll('[data-testid="working-set-project"]')).toHaveLength(2)
    expect(bar.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('aria-selected'))).toEqual(['false', 'true'])
  })

  it('shows the task list as the place on a project page', async () => {
    const { bar } = await barAt('/projects/vega')

    expect(bar.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('true')
  })

  it.each(['/task/t1/description', '/task/t1/notes/n1', '/task/t2/plan', '/projects/vega', '/console'])('opens the shortcut’s Plan from %s', async (path) => {
    useConsoleStore().steps = [step('s1', T1, 'DONE'), step('s2', T1, 'OPEN'), step('s3', T2, 'DRAFT')]
    if (path === '/console') showTerminalOf(T1)
    const { bar, router } = await barAt(path)
    const key = bar.get('[data-testid="working-set-task-tally"]')
    expect(bar.findAll('[data-testid="working-set-task-tally"]')).toHaveLength(1)
    expect(key.attributes('aria-current')).toBeUndefined()

    await key.trigger('click', { button: 0 })
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/t1/plan')
    expect(markedTasks(bar)).toEqual(['true', 'false'])
    expect(key.attributes('aria-current')).toBe('page')
    expect(key.classes()).toContain('key-lit')

    await bar.get('[data-testid="working-set-task"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/t1/description')
    expect(key.attributes('aria-current')).toBeUndefined()
  })

  it('is hidden on other pages and while nothing is open', async () => {
    const { bar: settings } = await barAt('/settings')
    expect(settings.find('[data-testid="working-set-bar"]').exists()).toBe(false)

    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [], places: [] })
    await useWorkingSetStore().load()
    const { bar: empty } = await barAt('/task/t1/description')
    expect(empty.find('[data-testid="working-set-bar"]').exists()).toBe(false)
  })

  it('goes to the place last seen in a project when its tab is pressed', async () => {
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-project"]')[0]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t3')
  })

  it('goes to the task list of a project whose remembered task is no longer open', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: VEGA }, { taskId: T3, projectId: ATLAS }], places: [{ projectId: VEGA, taskId: T2 }] })
    await useWorkingSetStore().load()
    const { bar, router } = await barAt('/task/t3/description')

    await bar.findAll('[data-testid="working-set-project"]')[1]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/projects/vega')
  })

  it('follows where the server says the project lands when the task in view is closed', async () => {
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T2 })
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-task-close"]')[0]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t2')
  })

  it('closes a task that is not in view without leaving the page', async () => {
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T1 })
    const { bar, router } = await barAt('/task/t1/description')

    await bar.findAll('[data-testid="working-set-task-close"]')[1]?.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.id).toBe('t1')
    expect(useWorkingSetStore().isOpen(T2)).toBe(false)
  })

  describe('on the terminal view', () => {
    it('shows both rows and marks the selected task', async () => {
      showTerminalOf(T2)

      const { bar } = await barAt('/console')

      expect(bar.findAll('[data-testid="working-set-project"]')).toHaveLength(2)
      expect(markedTasks(bar)).toEqual(['false', 'true'])
    })

    it('is hidden on a console route without the terminal', async () => {
      useConsoleStore().selectTask(T2)

      const { bar } = await barAt('/console')

      expect(bar.find('[data-testid="working-set-bar"]').exists()).toBe(false)
    })

    it('moves the mark when the terminal switches to another task', async () => {
      showTerminalOf(T2)
      const { bar } = await barAt('/console')

      showTerminalOf(T1)
      await flushPromises()

      expect(markedTasks(bar)).toEqual(['true', 'false'])
    })

    it('marks no tab and adds nothing for a task that is not open', async () => {
      vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: VEGA }], places: [] })
      await useWorkingSetStore().load()
      showTerminalOf(T2)

      const { bar } = await barAt('/console')

      expect(markedTasks(bar)).toEqual(['false'])
      expect(bar.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('false')
      expect(useWorkingSetStore().isOpen(T2)).toBe(false)
    })

    it('goes back to the task page when the marked tab is pressed', async () => {
      showTerminalOf(T2)
      const { bar, router } = await barAt('/console')

      await bar.findAll('[data-testid="working-set-task"]')[1]?.trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.fullPath).toBe('/task/t2/description')
    })

    it('lands on the neighbour when the task shown is closed', async () => {
      vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T1 })
      showTerminalOf(T2)
      const { bar, router } = await barAt('/console')

      await bar.findAll('[data-testid="working-set-task-close"]')[1]?.trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.fullPath).toBe('/task/t1/description')
    })
  })

  describe('on the chat view', () => {
    it('shows both rows and marks the selected task', async () => {
      useConsoleStore().selectTask(T2)
      useConsoleStore().openChat()

      const { bar } = await barAt('/console')

      expect(bar.findAll('[data-testid="working-set-project"]')).toHaveLength(2)
      expect(markedTasks(bar)).toEqual(['false', 'true'])
    })

    it('shows the project of a chat on no task, marking no tab', async () => {
      useTerminalStore().terminals = [{ id: 'chat-9' as TerminalId, taskId: null, projectId: ATLAS, kind: 'CHAT', live: true } as Terminal]
      useConsoleStore().openProjectChat('chat-9' as TerminalId)

      const { bar } = await barAt('/console')

      expect(bar.get('[data-testid="working-set-tasks"]').attributes('aria-label')).toBe('Atlas tasks')
      expect(markedTasks(bar)).toEqual(['false'])
      expect(bar.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('false')
    })

    it('goes back to the task page when the marked tab is pressed', async () => {
      useConsoleStore().selectTask(T2)
      useConsoleStore().openChat()
      const { bar, router } = await barAt('/console')

      await bar.findAll('[data-testid="working-set-task"]')[1]?.trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.fullPath).toBe('/task/t2/description')
    })
  })

  describe('the terminal key of a task', () => {
    it.each(['/task/t2/description', '/task/t1/description', '/projects/vega'])('shows its terminal from %s', async (path) => {
      liveTerminalOn(T1)
      const { bar, router } = await barAt(path)

      await bar.get('[data-testid="working-set-task-terminal"]').trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('console')
      expect(useConsoleStore().selectedTaskId).toBe(T1)
      expect(useConsoleStore().paneFocus).toBe('terminal')
      expect(useTerminalStore().activeTerminalId).toBe('term-t1')
      expect(bar.get('[data-testid="working-set-task-terminal"]').attributes('aria-pressed')).toBe('true')
    })

    it('shows its terminal from the terminal view of another task', async () => {
      liveTerminalOn(T1)
      showTerminalOf(T2)
      const { bar } = await barAt('/console')

      await bar.get('[data-testid="working-set-task-terminal"]').trigger('click')
      await flushPromises()

      expect(useConsoleStore().selectedTaskId).toBe(T1)
      expect(useTerminalStore().activeTerminalId).toBe('term-t1')
      expect(markedTasks(bar)).toEqual(['true', 'false'])
    })

    it('leaves the terminal on screen alone on a second press', async () => {
      liveTerminalOn(T1)
      showTerminalOf(T1)
      const select = vi.spyOn(useConsoleStore(), 'selectTask')
      const { bar } = await barAt('/console')

      await bar.get('[data-testid="working-set-task-terminal"]').trigger('click')

      expect(select).not.toHaveBeenCalled()
    })

    it('leaves when the session ends', async () => {
      liveTerminalOn(T1)
      const { bar } = await barAt('/task/t2/description')

      useTerminalStore().markEnded('term-t1' as TerminalId)
      await flushPromises()

      expect(bar.find('[data-testid="working-set-task-terminal"]').exists()).toBe(false)
    })
  })

  describe('the chat key of a task', () => {
    const CHAT_KEY = '[data-testid="working-set-task-chat"]'

    it('gives each task the key of its own session', async () => {
      useTerminalStore().terminals = [
        { id: 'chat-t1' as TerminalId, taskId: T1, kind: 'CHAT', live: true } as Terminal,
        { id: 'term-t2' as TerminalId, taskId: T2, kind: 'TERMINAL', live: true } as Terminal
      ]

      const { bar } = await barAt('/task/t2/description')

      expect(bar.findAll('[aria-pressed]').map((key) => key.attributes('aria-label'))).toEqual(['Open the chat of t1', 'Open the terminal of t2'])
    })

    it.each(['/task/t2/description', '/task/t1/description', '/projects/vega'])('shows its chat from %s', async (path) => {
      liveChatOn(T1)
      const { bar, router } = await barAt(path)

      await bar.get(CHAT_KEY).trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('console')
      expect(useConsoleStore().selectedTaskId).toBe(T1)
      expect(useConsoleStore().paneFocus).toBe('chat')
      expect(useChatStore().activeChatId).toBe('chat-t1')
      expect(bar.get(CHAT_KEY).attributes('aria-pressed')).toBe('true')
    })

    it('leaves the chat on screen alone on a second press', async () => {
      liveChatOn(T1)
      showChatOf(T1)
      const select = vi.spyOn(useConsoleStore(), 'selectTask')
      const { bar } = await barAt('/console')

      await bar.get(CHAT_KEY).trigger('click')

      expect(select).not.toHaveBeenCalled()
    })

    it('stays, lit, on a chat that ends while it is on screen, until the list is read again', async () => {
      liveChatOn(T1)
      showChatOf(T1)
      const { bar } = await barAt('/console')

      useTerminalStore().markEnded('chat-t1' as TerminalId)
      await flushPromises()
      expect(bar.get(CHAT_KEY).attributes('aria-pressed')).toBe('true')

      useTerminalStore().terminals = []
      await flushPromises()
      expect(bar.find(CHAT_KEY).exists()).toBe(false)
    })

    it('leaves when a chat ends out of sight and the console hears it', async () => {
      liveChatOn(T1)
      const { bar } = await barAt('/task/t2/description')

      useTerminalStore().markEnded('chat-t1' as TerminalId)
      await flushPromises()

      expect(bar.find(CHAT_KEY).exists()).toBe(false)
    })
  })
})
