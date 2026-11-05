import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import DescriptionPane from '@/features/catalog/components/DescriptionPane.vue'
import DescriptionEmptyState from '@/features/catalog/components/DescriptionEmptyState.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import * as componentsApi from '@/features/project/api/project-components.api'
import { useConsoleStore } from '@/stores/console.store'
import type { Task, TaskStepState } from '@/features/catalog/model/catalog'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId, TaskId } from '@/model/branded'

/** The size chip measures on its own; here it only has to not reach the network. */
vi.mock('@/features/context/api/context.api', () => ({
  fetchContextSize: vi.fn(async () => ({ characters: 3500, estimatedTokens: 1000, parts: [] }))
}))

vi.mock('@/features/project/api/project-components.api')
vi.mock('@/features/feedback/api/feedback.api', () => ({ fetchFeedback: vi.fn(async () => []) }))

/**
 * The description's own review bar, shown only while the task has no checklist. It has to give
 * the same OPEN -> RUNNING -> CLAIMED -> DONE affordances a step does: while a session is on a
 * stepless task the bar still offers Accept, because otherwise the running state has no console
 * exit and the wrapup path is the only way forward.
 */
const rekall = 'p1' as ProjectId
const taskId = 't1' as TaskId

function makeTask(reviewState: TaskStepState, wrapup: Partial<Task> = {}): Task {
  return {
    id: taskId,
    label: 'application-improvements',
    title: 'Application improvements',
    status: 'IN_PROGRESS',
    description: null,
    projectId: rekall,
    projectLabel: 'rekall',
    projectTitle: 'Rekall',
    companyName: 'vforge',
    projectRepoFolder: null,
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0, draftStepCount: 0,
    hasWrapup: false,
    reviewState,
    reviewActive: true,
    claimedAt: null,
    acceptedAt: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    graphLaidAt: null,
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null, quick: false, autoAccepted: false, dependsOn: [],
    anchor: 'project:rekall task:application-improvements',
    updatedAt: '2026-09-09T10:00:00Z',
    ...wrapup
  }
}

function seed(reviewState: TaskStepState, wrapup: Partial<Task> = {}) {
  const store = useConsoleStore()
  store.tasks = [makeTask(reviewState, wrapup)]
  store.selectedTaskId = taskId
  store.isLoading = false
  store.acceptTask = vi.fn().mockResolvedValue(undefined)
  store.saveTaskDescription = vi.fn().mockResolvedValue(undefined)
  return store
}

let pinia: Pinia

function threadEntry(extra: Record<string, unknown>): Record<string, unknown> {
  return { taskId, kind: 'IMPLEMENTATION', quotedMarkdown: null, bodyMarkdown: 'x', replyMarkdown: null, createdAt: '2026-11-04T10:00:00Z', appliedAt: null, pass: null, takenAt: null, followsSeq: null, ...extra }
}

function render() {
  return mount(DescriptionPane, {
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        OpenTerminalButton: true,
        LaunchClaudeCodeButton: true
      }
    }
  })
}

describe('DescriptionPane review bar', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('offers no Accept while a session has the stepless task running, and says what is running', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([threadEntry({ id: 'f1', seq: 4, state: 'RUNNING', pass: 2, takenAt: '2026-11-04T10:30:00Z' }), threadEntry({ id: 'f2', seq: 5, state: 'RUNNING', pass: 2, takenAt: '2026-11-04T10:30:00Z' })] as never)
    seed('RUNNING')
    const wrapper = render()
    await flushPromises()

    const bar = wrapper.get('[data-testid="description-review-bar"]')
    expect(wrapper.find('[data-testid="description-accept"]').exists()).toBe(false)
    expect(bar.text()).toContain('Pass 2 is running on #4, #5')
  })

  it('says a session is on the task when no pass is running, still without Accept', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([])
    seed('RUNNING')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="description-accept"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="description-review-bar"]').text()).toContain('A session is working on this task')
  })

  it('keeps Accept as the main action once a wrapup has claimed the task, and points corrections at the thread', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([])
    const store = seed('CLAIMED')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.get('[data-testid="description-accept"]').text()).toBe('Accept')
    expect(wrapper.get('[data-testid="description-review-bar"]').text()).toContain('feedback below')
    expect(wrapper.find('[data-testid="description-review-bar"] [data-testid="apply-feedback"]').exists()).toBe(false)

    await wrapper.get('[data-testid="description-accept"]').trigger('click')
    expect(store.acceptTask).toHaveBeenCalledWith(taskId)
  })

  it('lays out what the last pass did: a reply per entry and the commits logged since it started', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([
      threadEntry({ id: 'f1', seq: 1, state: 'APPLIED', pass: 1, takenAt: '2026-11-04T09:00:00Z', replyMarkdown: 'Old fix.', appliedAt: '2026-11-04T09:30:00Z' }),
      threadEntry({ id: 'f2', seq: 2, state: 'APPLIED', pass: 2, takenAt: '2026-11-04T10:00:00Z', replyMarkdown: 'Fixed the totals.\nMore.', appliedAt: '2026-11-04T10:30:00Z' })
    ] as never)
    const store = seed('CLAIMED')
    store.commitReferences = [
      { id: 'c1', taskId, stepId: null, stepTitle: null, commitHash: 'aaaaaaa111', comment: 'before the pass', inContext: false, createdAt: '2026-11-04T08:00:00Z' },
      { id: 'c2', taskId, stepId: null, stepTitle: null, commitHash: 'bbbbbbb222', comment: 'fix the totals', inContext: false, createdAt: '2026-11-04T10:20:00Z' }
    ] as never
    const wrapper = render()
    await flushPromises()

    const summary = wrapper.get('[data-testid="pass-summary"]')
    expect(summary.get('[data-testid="pass-summary-title"]').text()).toBe('What pass 2 did')
    expect(summary.findAll('[data-testid="pass-summary-entry"]')).toHaveLength(1)
    expect(summary.text()).toContain('Fixed the totals.')
    expect(summary.findAll('[data-testid="pass-summary-commit"]').map((commit) => commit.text())).toEqual(['bbbbbbbfix the totals'])
  })

  it('makes Run revision the main action and turns Accept into Accept anyway while entries wait', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([threadEntry({ id: 'f1', seq: 1, state: 'PENDING' }), threadEntry({ id: 'f2', seq: 2, state: 'PENDING', kind: 'PLAN' })] as never)
    const store = seed('CLAIMED')
    const wrapper = render()
    await flushPromises()

    const bar = wrapper.get('[data-testid="description-review-bar"]')
    expect(bar.find('[data-testid="apply-feedback"]').exists()).toBe(true)
    const accept = bar.get('[data-testid="description-accept"]')
    expect(accept.text()).toBe('Accept anyway')
    expect(accept.attributes('title')).toBe('2 entries have not been run')
    expect(bar.text()).toContain('2 entries are waiting for the next run')

    await accept.trigger('click')
    expect(store.acceptTask).toHaveBeenCalledWith(taskId)
  })

  it('shows no review bar while the stepless task is still open', async () => {
    seed('OPEN')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="description-review-bar"]').exists()).toBe(false)
  })
})

describe('DescriptionPane component completion', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it("hands the project's components to the editor being written in, so [[ has names to offer", async () => {
    const auth: ProjectComponent = {
      id: 'c1' as ProjectComponentId,
      projectId: rekall,
      name: 'auth-service',
      folder: '/work/auth',
      description: 'Sessions',
      position: 0,
      createdAt: '2026-10-08T10:00:00Z',
      updatedAt: '2026-10-08T10:00:00Z'
    }
    vi.mocked(componentsApi.fetchProjectComponents).mockResolvedValue([auth])
    seed('OPEN')
    const wrapper = render()
    await flushPromises()

    wrapper.getComponent(DescriptionEmptyState).vm.$emit('write')
    await flushPromises()

    expect(wrapper.getComponent({ name: 'AppMarkdownEditor' }).props('components')).toEqual([auth])
  })
})

describe('DescriptionPane mode', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('opens the next task in Write when the reader left the description in Write', async () => {
    const store = seed('OPEN', { description: 'The first brief.' })
    const wrapper = render()
    await flushPromises()
    expect(wrapper.getComponent(AppModeToggle).props('modelValue')).toBe('read')

    wrapper.getComponent(AppModeToggle).vm.$emit('update:modelValue', 'write')
    const other = { ...makeTask('OPEN'), id: 't2' as TaskId, description: 'The second brief.' }
    store.tasks = [...store.tasks, other]
    store.selectedTaskId = other.id
    await flushPromises()

    expect(wrapper.getComponent(AppModeToggle).props('modelValue')).toBe('write')
  })
})

describe('DescriptionPane thread', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('reads the thread of the task in view and shows it under the description', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    vi.mocked(fetchFeedback).mockResolvedValue([
      { id: 'f1', taskId, seq: 1, kind: 'IMPLEMENTATION', state: 'PENDING', quotedMarkdown: null, bodyMarkdown: 'The totals are wrong', replyMarkdown: null, createdAt: '2026-11-04T10:00:00Z', appliedAt: null, pass: null, takenAt: null, followsSeq: null }
    ] as never)
    seed('CLAIMED')
    const wrapper = render()
    await flushPromises()

    expect(fetchFeedback).toHaveBeenCalledWith(taskId)
    expect(wrapper.find('[data-testid="feedback-dock"]').exists()).toBe(true)
    expect(wrapper.findAll('[data-testid="feedback-entry"]')).toHaveLength(1)
  })

  it('discards a waiting revision from the thread, which asks the store to restore the text', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    const revision = { id: 'f1', taskId, seq: 1, kind: 'REVISION', state: 'PENDING', quotedMarkdown: 'before', bodyMarkdown: 'after', replyMarkdown: null, createdAt: '2026-11-04T10:00:00Z', appliedAt: null, pass: null, takenAt: null, followsSeq: null }
    vi.mocked(fetchFeedback).mockResolvedValue([revision] as never)
    const store = seed('CLAIMED')
    const discard = vi.spyOn(store, 'discardRevision').mockResolvedValue()
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[data-testid="revision-discard"]').trigger('click')

    expect(discard).toHaveBeenCalledWith(expect.objectContaining({ id: 'f1', kind: 'REVISION' }))
  })

  it('has no revision banner and one Run revision button, in the header of the dock', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    const waiting = { id: 'f1', taskId, seq: 1, kind: 'REVISION', state: 'PENDING', quotedMarkdown: 'before', bodyMarkdown: 'after', replyMarkdown: null, createdAt: '2026-11-04T10:00:00Z', appliedAt: null, pass: null, takenAt: null, followsSeq: null }
    vi.mocked(fetchFeedback).mockResolvedValue([waiting] as never)
    seed('OPEN')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="revision-banner"]').exists()).toBe(false)
    expect(wrapper.findAll('[data-testid="apply-feedback"]')).toHaveLength(1)
    expect(wrapper.find('[data-testid="feedback-dock"] [data-testid="apply-feedback"]').exists()).toBe(true)
  })

  it('opens the dock by itself when an entry waits, and keeps it folded when everything was applied', async () => {
    const { fetchFeedback } = await import('@/features/feedback/api/feedback.api')
    const base = { taskId, kind: 'IMPLEMENTATION', quotedMarkdown: null, bodyMarkdown: 'x', createdAt: '2026-11-04T10:00:00Z', pass: null, takenAt: null, followsSeq: null }
    vi.mocked(fetchFeedback).mockResolvedValue([{ ...base, id: 'f1', seq: 1, state: 'APPLIED', replyMarkdown: 'Done.', appliedAt: '2026-11-04T11:00:00Z' }] as never)
    seed('CLAIMED')
    const folded = render()
    await flushPromises()
    expect(folded.find('[data-testid="feedback-thread"]').exists()).toBe(false)

    vi.mocked(fetchFeedback).mockResolvedValue([{ ...base, id: 'f2', seq: 1, state: 'PENDING', replyMarkdown: null, appliedAt: null }] as never)
    await useConsoleStore().loadFeedback(taskId)
    await flushPromises()

    expect(folded.find('[data-testid="feedback-thread"]').exists()).toBe(true)
  })
})
