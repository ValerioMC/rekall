import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import DescriptionRewriteButton from '@/features/catalog/components/DescriptionRewriteButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TerminalId } from '@/model/branded'

const openTerminal = vi.fn()
const sendTerminalInput = vi.fn()
const fetchTask = vi.fn()

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...args: unknown[]) => openTerminal(...args),
  closeTerminal: vi.fn(),
  sendTerminalInput: (...args: unknown[]) => sendTerminalInput(...args)
}))

vi.mock('@/features/catalog/api/catalog.api', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  fetchTask: (...args: unknown[]) => fetchTask(...args)
}))

const TASK = 't1' as TaskId
const ANCHOR = 'project:vega task:report-builder'
const ROUGH = 'make the report exportable pls with pdf'
const POLISHED = 'Make the report exportable, with PDF.'

function task(description: string | null): Task {
  return {
    id: TASK,
    code: 'report-builder', label: 'report-builder',
    title: 'Report builder',
    status: 'IN_PROGRESS',
    description,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'vforge',
    projectRepoFolder: '/code/vega',
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: 'OPEN',
    reviewActive: false,
    claimedAt: null,
    acceptedAt: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    graphLaidAt: null,
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null,
    quick: false,
    autoAccepted: false,
    dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
    anchor: ANCHOR,
    updatedAt: '2026-11-11T10:00:00Z'
  }
}

function liveTerminal(): Terminal {
  return {
    id: 'term-1' as TerminalId,
    projectId: 'p1' as ProjectId,
    taskId: TASK,
    stepId: null,
    anchors: ANCHOR,
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskCode: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'REWRITE_FIX',
    skipPermissions: true,
    model: null,
    effort: null,
    kind: 'TERMINAL',
    live: true,
    awaitingPermission: false,
    startedAt: '2026-11-11T10:00:00Z',
    lastActivityAt: '2026-11-11T10:00:00Z',
    connections: []
  }
}

function pane(description: string | null = ROUGH) {
  const store = useConsoleStore()
  store.tasks = [task(description)]
  store.selectedTaskId = TASK
  const saveTaskDescription = vi.fn().mockResolvedValue(undefined)
  store.saveTaskDescription = saveTaskDescription
  const wrapper = mount(DescriptionRewriteButton, { props: { taskId: TASK, anchor: ANCHOR, folder: '/code/vega' }, attachTo: document.body })
  return { wrapper, store, saveTaskDescription }
}

function busy(wrapper: ReturnType<typeof pane>['wrapper']) {
  return wrapper.find('[data-testid="description-rewrite-busy"]')
}

async function pressFix(wrapper: ReturnType<typeof pane>['wrapper']): Promise<void> {
  await wrapper.get('[data-testid="description-rewrite-fix"]').trigger('click')
}

async function advance(milliseconds: number): Promise<void> {
  await vi.advanceTimersByTimeAsync(milliseconds)
  await flushPromises()
}

describe('the rewrite key of the description pane', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.useFakeTimers()
    openTerminal.mockResolvedValue(liveTerminal())
    sendTerminalInput.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  it('goes from launching to working to done to idle as the session rewrites the description', async () => {
    fetchTask.mockResolvedValueOnce(task(ROUGH)).mockResolvedValue(task(POLISHED))
    const { wrapper } = pane()

    await pressFix(wrapper)
    expect(busy(wrapper).attributes('data-phase')).toBe('launching')
    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'REWRITE_FIX' }))

    await advance(400)
    expect(busy(wrapper).attributes('data-phase')).toBe('working')

    await advance(4000)
    expect(busy(wrapper).attributes('data-phase')).toBe('done')
    expect(useConsoleStore().tasks[0]?.description).toBe(POLISHED)

    await advance(1300)
    expect(busy(wrapper).exists()).toBe(false)
    expect(wrapper.find('[data-testid="description-rewrite-fix"]').exists()).toBe(true)
  })

  it('types the rewrite line into the live session of the task instead of opening another', async () => {
    useTerminalStore().upsert(liveTerminal())
    fetchTask.mockResolvedValue(task(POLISHED))
    const { wrapper } = pane()

    await pressFix(wrapper)
    await advance(400)

    expect(openTerminal).not.toHaveBeenCalled()
    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} rewrite fix\r`)
  })

  it('restructures from the menu', async () => {
    fetchTask.mockResolvedValue(task(POLISHED))
    const { wrapper } = pane()

    await wrapper.get('[data-testid="split-button-arrow"]').trigger('click')
    document.body.querySelector<HTMLElement>('[data-testid="split-button-option-RESTRUCTURE"]')?.click()
    await advance(400)

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'REWRITE_RESTRUCTURE' }))
  })

  it('leaves the description untouched, with no Undo, when the session cannot start', async () => {
    openTerminal.mockRejectedValue(new Error('no claude'))
    const { wrapper } = pane()

    await pressFix(wrapper)
    await advance(400)

    expect(busy(wrapper).exists()).toBe(false)
    expect(fetchTask).not.toHaveBeenCalled()
    expect(useConsoleStore().tasks[0]?.description).toBe(ROUGH)
    expect(wrapper.find('[data-testid="description-rewrite-undo"]').exists()).toBe(false)
  })

  it('stops waiting when the session ends without changing the description', async () => {
    fetchTask.mockResolvedValue(task(ROUGH))
    const { wrapper } = pane()

    await pressFix(wrapper)
    await advance(400)
    useTerminalStore().markEnded('term-1' as TerminalId)
    await advance(2100)

    expect(busy(wrapper).exists()).toBe(false)
    expect(wrapper.find('[data-testid="description-rewrite-undo"]').exists()).toBe(false)
  })

  it('offers Undo once the text is replaced and writes the old text back through the store', async () => {
    fetchTask.mockResolvedValue(task(POLISHED))
    const { wrapper, saveTaskDescription } = pane()
    await pressFix(wrapper)
    await advance(400 + 2100 + 1300)

    await wrapper.get('[data-testid="description-rewrite-undo"]').trigger('click')
    await flushPromises()

    expect(saveTaskDescription).toHaveBeenCalledWith(TASK, ROUGH)
    expect(wrapper.find('[data-testid="description-rewrite-undo"]').exists()).toBe(false)
  })

  it('drops the Undo once the person edits the description', async () => {
    fetchTask.mockResolvedValue(task(POLISHED))
    const { wrapper, store } = pane()
    await pressFix(wrapper)
    await advance(400 + 2100 + 1300)
    expect(wrapper.find('[data-testid="description-rewrite-undo"]').exists()).toBe(true)

    store.saveState = 'unsaved'
    await flushPromises()

    expect(wrapper.find('[data-testid="description-rewrite-undo"]').exists()).toBe(false)
  })

  it('is disabled while the description is blank or has unsaved changes', async () => {
    const blank = pane(null)
    expect(blank.wrapper.get('[data-testid="description-rewrite-fix"]').attributes('disabled')).toBeDefined()
    blank.wrapper.unmount()

    setActivePinia(createPinia())
    const unsaved = pane()
    unsaved.store.saveState = 'unsaved'
    await flushPromises()
    expect(unsaved.wrapper.get('[data-testid="description-rewrite-fix"]').attributes('disabled')).toBeDefined()
  })
})
