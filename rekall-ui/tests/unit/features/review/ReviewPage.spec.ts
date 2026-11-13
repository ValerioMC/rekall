import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'
import ReviewPage from '@/features/review/components/ReviewPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/features/adherence/api/adherence.api', () => ({
  fetchTaskAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, deviationSteps: [], visits: []
  })),
  fetchProjectAdherence: vi.fn(async () => ({}))
}))

const stub = { template: '<div />' }

function makeTask(id: string, overrides: Partial<Task> = {}): Task {
  return {
    id: id as TaskId, code: id.toUpperCase(), label: id, title: `Task ${id}`, status: 'IN_PROGRESS', description: null,
    projectId: 'p1' as ProjectId, projectLabel: 'vega', projectTitle: 'Vega', companyName: 'Acme', projectRepoFolder: '/work/vega',
    documentCount: 0, stepCount: 1, stepsDone: 0, draftStepCount: 0, hasWrapup: false, reviewState: 'OPEN', reviewActive: false,
    claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: '2026-10-01T08:00:00Z',
    releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null,
    trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null,
    pullRequestFailure: null, pullRequestMergedAt: null, anchor: `task:${id.toUpperCase()}`, updatedAt: '2026-10-01T10:00:00Z',
    ...overrides
  }
}

function makeStep(id: string, taskId: string, claimedAt: string, state: TaskStep['state'] = 'CLAIMED'): TaskStep {
  return {
    id: id as TaskStepId, taskId: taskId as TaskId, title: `Step ${id}`, bodyMarkdown: null, state, done: false, runningAt: null,
    claimedAt, doneAt: null, position: 0, flowX: null, flowY: null, maxVisits: null, startsSession: false, connectionId: null,
    passes: [], links: [], visits: [], createdAt: '2026-10-01T07:00:00Z', updatedAt: '2026-10-01T07:00:00Z'
  }
}

let pinia: Pinia
let router: Router

function seed() {
  const store = useConsoleStore()
  store.tasks = [
    makeTask('rk-1'),
    makeTask('rk-2'),
    makeTask('rk-3', { stepCount: 0, reviewActive: true, reviewState: 'CLAIMED', claimedAt: '2026-10-01T09:30:00Z' })
  ]
  store.steps = [makeStep('s2', 'rk-2', '2026-10-01T09:10:00Z'), makeStep('s1', 'rk-1', '2026-10-01T09:00:00Z')]
  store.isLoading = false
  store.acceptStep = vi.fn().mockImplementation(async (id: string) => {
    store.steps = store.steps.map((step) => (step.id === id ? { ...step, state: 'DONE', done: true } : step))
  })
  store.sendBackStep = vi.fn().mockResolvedValue(undefined)
  store.acceptTask = vi.fn().mockResolvedValue(undefined)
  store.sendBackTask = vi.fn().mockResolvedValue(undefined)
  return store
}

async function mountAt(path: string) {
  await router.push(path)
  const wrapper = mount(ReviewPage, {
    attachTo: document.body,
    global: {
      plugins: [pinia, router],
      stubs: {
        AppMarkdownEditor: true,
        OpenTerminalButton: true,
        LogCommitButton: true,
        CommitReferenceList: true,
        PlanAdherencePanel: true
      }
    }
  })
  await flushPromises()
  return wrapper
}

describe('ReviewPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'work', component: stub },
        { path: '/review/:key?', name: 'review', component: stub },
        { path: '/task/:id/:tab?/:noteId?', name: 'task', component: stub }
      ]
    })
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('lists every claim, the longest waiting first, and the count is the queue’s', async () => {
    const store = seed()
    const page = await mountAt('/review')

    const titles = page.findAll('[data-testid="review-item"]').map((row) => row.text())
    expect(titles[0]).toContain('Step s1')
    expect(titles[1]).toContain('Step s2')
    expect(titles[2]).toContain('Task rk-3')
    expect(titles[2]).toContain('whole task')
    expect(page.get('[data-testid="review-count"]').text()).toBe(String(store.reviewQueue.length))
  })

  it('opens the first item and puts its key in the URL', async () => {
    seed()
    const page = await mountAt('/review')

    expect(router.currentRoute.value.params.key).toBe('s1')
    expect(page.get('[data-testid="decision-title"]').text()).toBe('Step s1')
    expect(page.get('[data-testid="review-task-link"]').text()).toBe('Task rk-1')
  })

  it('opens the item a link names', async () => {
    seed()
    const page = await mountAt('/review/s2')

    expect(page.get('[data-testid="decision-title"]').text()).toBe('Step s2')
  })

  it('shows a task handed in whole with its own decision', async () => {
    seed()
    const page = await mountAt('/review/rk-3')

    expect(page.find('[data-testid="handed-in-panel"]').exists()).toBe(true)
  })

  it('accepts a claim and moves on to the next', async () => {
    const store = seed()
    const page = await mountAt('/review/s1')

    await page.get('[data-testid="step-accept"]').trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s1')
    expect(router.currentRoute.value.params.key).toBe('s2')
    expect(page.findAll('[data-testid="review-item"]')).toHaveLength(2)
  })

  it('sends a claim back and moves on once the session has taken it', async () => {
    const store = seed()
    store.sendBackStep = vi.fn().mockImplementation(async (id: string) => {
      store.steps = store.steps.map((step) => (step.id === id ? { ...step, state: 'OPEN' } : step))
    })
    const page = await mountAt('/review/s2')

    await page.get('[data-testid="step-send-back"]').trigger('click')
    await flushPromises()

    expect(store.sendBackStep).toHaveBeenCalledWith('s2')
    expect(router.currentRoute.value.params.key).toBe('rk-3')
  })

  it('leaves nothing selected after a decision when auto-advance is off', async () => {
    const store = seed()
    usePreferencesStore().reviewAutoAdvance = false
    const page = await mountAt('/review/s1')

    await page.get('[data-testid="step-accept"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.key).toBeUndefined()
    expect(page.find('[data-testid="review-pick-next"]').exists()).toBe(true)
    expect(page.findAll('[data-testid="review-item"]')).toHaveLength(2)
    await page.findAll('[data-testid="review-item"]')[0]!.trigger('click')
    await flushPromises()
    expect(page.find('[data-testid="review-pick-next"]').exists()).toBe(false)
    expect(store.acceptStep).toHaveBeenCalledOnce()
  })

  it('asks before sending a claim back when Settings says so', async () => {
    const store = seed()
    store.sendBackStep = vi.fn().mockResolvedValue(undefined)
    usePreferencesStore().reviewConfirmSendBack = true
    const page = await mountAt('/review/s2')

    await page.get('[data-testid="step-send-back"]').trigger('click')
    await flushPromises()
    expect(store.sendBackStep).not.toHaveBeenCalled()
    expect(useSendBackGuardStore().pending).not.toBeNull()

    await useSendBackGuardStore().confirm()
    expect(store.sendBackStep).toHaveBeenCalledWith('s2')
  })

  it('says nothing waits once the queue is empty', async () => {
    const store = seed()
    store.tasks = [makeTask('rk-1')]
    store.steps = []
    const page = await mountAt('/review')

    expect(page.get('[data-testid="review-empty"]').text()).toContain('Nothing waits for you.')
    expect(router.currentRoute.value.params.key).toBeUndefined()
  })
})
