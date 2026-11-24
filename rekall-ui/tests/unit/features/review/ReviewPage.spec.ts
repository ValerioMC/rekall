import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'
import ReviewPage from '@/features/review/components/ReviewPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { Wrapup } from '@/features/wrapup/model/wrapup'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/features/adherence/api/adherence.api', () => ({
  fetchTaskAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, deviationSteps: [], visits: []
  })),
  fetchProjectAdherence: vi.fn(async () => ({}))
}))

vi.mock('@/features/check/api/checks.api', () => ({
  fetchTaskCheckResults: async (taskId: string) =>
    taskId === 'rk-1'
      ? [
          { id: 'c1', checkId: 'k1', taskId, stepId: 's1', label: 'cargo test', command: 'cargo test', passed: false, timedOut: false, exitCode: 101, durationMs: 900, outputTail: 'assertion failed: left 8000, right 8003', ranAt: '2026-10-01T09:05:00Z' },
          { id: 'c2', checkId: 'k2', taskId, stepId: 's1', label: 'cargo clippy', command: 'cargo clippy', passed: true, timedOut: false, exitCode: 0, durationMs: 400, outputTail: '', ranAt: '2026-10-01T09:06:00Z' }
        ]
      : [],
  fetchChecks: async () => []
}))

vi.mock('@/features/deviations/api/deviations.api', () => ({
  fetchDeviations: async (taskId: string) =>
    taskId === 'rk-1' ? [{ id: 'd1', taskId, stepNumber: 1, stepTitle: 'Step s1', bodyMarkdown: 'Counted characters, not bytes.', createdAt: '2026-10-01T09:00:00Z', updatedAt: '2026-10-01T09:00:00Z' }] : []
}))

const stub = { template: '<div />' }
const markdown = { props: ['modelValue'], template: '<div data-testid="markdown">{{ modelValue }}</div>' }

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
    id: id as TaskStepId, taskId: taskId as TaskId, title: `Step ${id}`, bodyMarkdown: `Brief of ${id}`, state, done: false, runningAt: null,
    claimedAt, doneAt: null, position: 0, flowX: null, flowY: null, maxVisits: null, startsSession: false, connectionId: null,
    passes: [], links: [], visits: [], createdAt: '2026-10-01T07:00:00Z', updatedAt: '2026-10-01T07:00:00Z'
  }
}

let pinia: Pinia
let router: Router
const mounted: VueWrapper[] = []

function seed() {
  const store = useConsoleStore()
  store.tasks = [
    makeTask('rk-1'),
    makeTask('rk-2'),
    makeTask('rk-3', { stepCount: 0, reviewActive: true, reviewState: 'CLAIMED', claimedAt: '2026-10-01T09:30:00Z' })
  ]
  store.steps = [makeStep('s2', 'rk-2', '2026-10-01T09:10:00Z'), makeStep('s1', 'rk-1', '2026-10-01T09:00:00Z')]
  store.wrapups = [{ taskId: 'rk-3' as TaskId, bodyMarkdown: 'Resume reads the stored session id.' } as Wrapup]
  store.commitReferences = [
    { id: 'cr1', taskId: 'rk-1' as TaskId, stepId: 's1' as TaskStepId, commitHash: '9f3c1abdeadbeef', comment: 'feat: cap the wrapup', createdAt: '2026-10-01T09:02:00Z' } as CommitReference
  ]
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
        AppMarkdownEditor: markdown,
        StepPassHistory: true
      }
    }
  })
  await flushPromises()
  mounted.push(wrapper)
  return wrapper
}

describe('ReviewPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/projects', name: 'projects', component: stub },
        { path: '/review/:key?', name: 'review', component: stub },
        { path: '/task/:id/:tab?/:noteId?', name: 'task', component: stub }
      ]
    })
  })

  afterEach(() => {
    mounted.splice(0).forEach((wrapper) => wrapper.unmount())
    document.body.innerHTML = ''
  })

  function press(key: string, target: EventTarget = document.body): void {
    target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
  }

  it('lists every claim, the longest waiting first, and the count is the queue’s', async () => {
    const store = seed()
    const page = await mountAt('/review')

    const titles = page.findAll('[data-testid="review-item"]').map((card) => card.text())
    expect(titles[0]).toContain('Step s1')
    expect(titles[1]).toContain('Step s2')
    expect(titles[2]).toContain('Task rk-3')
    expect(page.findAll('[data-testid="review-item-line"]')[2]!.text()).toContain('whole task')
    expect(page.get('[data-testid="review-count"]').text()).toBe(String(store.reviewQueue.length))
  })

  it('opens the first claim and puts its key in the URL, the others staying a line', async () => {
    seed()
    const page = await mountAt('/review')

    expect(router.currentRoute.value.params.key).toBe('s1')
    expect(page.findAll('[data-testid="review-claim-body"]')).toHaveLength(1)
    expect(page.findAll('[data-testid="review-item"]')[0]!.attributes('data-open')).toBe('true')
    expect(page.find('[data-testid="review-task-link"]').exists()).toBe(true)
  })

  it('opens the claim a link names', async () => {
    seed()
    const page = await mountAt('/review/s2')

    expect(page.findAll('[data-testid="review-item"]')[1]!.attributes('data-open')).toBe('true')
  })

  it('opens a claim when its title is pressed', async () => {
    seed()
    const page = await mountAt('/review/s1')

    await page.findAll('[data-testid="review-item-title"]')[1]!.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.key).toBe('s2')
  })

  it('says what was found on a claim before it is opened: the failed check, the commit and the deviation', async () => {
    seed()
    const page = await mountAt('/review/s2')

    const facts = page.findAll('[data-testid="review-item"]')[0]!.get('[data-testid="review-facts"]').text()
    expect(facts).toContain('1 of 2 checks failed')
    expect(facts).toContain('1 commit')
    expect(facts).toContain('Deviation')
  })

  it('shows the brief, the failed check with its output, the commit and the deviation on the open claim', async () => {
    seed()
    const page = await mountAt('/review/s1')

    const body = page.get('[data-testid="review-claim-body"]')
    expect(body.get('[data-testid="review-brief"]').text()).toBe('Brief of s1')
    expect(body.get('[data-testid="review-check-output"]').text()).toContain('left 8000, right 8003')
    expect(body.get('[data-testid="review-commits"]').text()).toContain('9f3c1ab')
    expect(body.get('[data-testid="review-commits"]').text()).toContain('feat: cap the wrapup')
    expect(body.get('[data-testid="review-deviation"]').text()).toContain('Counted characters, not bytes.')
  })

  it('shows the wrapup of a task handed in whole in place of a brief', async () => {
    seed()
    const page = await mountAt('/review/rk-3')

    expect(page.get('[data-testid="review-brief"]').text()).toBe('Resume reads the stored session id.')
    expect(page.findAll('[data-testid="review-accept"]')[2]!.text()).toContain('Accept task')
  })

  it('accepts a claim and moves on to the next', async () => {
    const store = seed()
    const page = await mountAt('/review/s1')

    await page.get('[data-testid="review-accept"]').trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s1')
    expect(router.currentRoute.value.params.key).toBe('s2')
    expect(page.findAll('[data-testid="review-item"]')).toHaveLength(2)
  })

  it('accepts a claim that is not open without opening it', async () => {
    const store = seed()
    const page = await mountAt('/review/s1')

    await page.findAll('[data-testid="review-accept"]')[1]!.trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s2')
  })

  it('accepts a task handed in whole', async () => {
    const store = seed()
    const page = await mountAt('/review/rk-3')

    await page.findAll('[data-testid="review-accept"]')[2]!.trigger('click')
    await flushPromises()

    expect(store.acceptTask).toHaveBeenCalledWith('rk-3')
  })

  it('sends a claim back and moves on once the session has taken it', async () => {
    const store = seed()
    store.sendBackStep = vi.fn().mockImplementation(async (id: string) => {
      store.steps = store.steps.map((step) => (step.id === id ? { ...step, state: 'OPEN' } : step))
    })
    const page = await mountAt('/review/s2')

    await page.findAll('[data-testid="review-send-back"]')[1]!.trigger('click')
    await flushPromises()

    expect(store.sendBackStep).toHaveBeenCalledWith('s2')
    expect(router.currentRoute.value.params.key).toBe('rk-3')
  })

  it('leaves nothing open after a decision when auto-advance is off', async () => {
    const store = seed()
    usePreferencesStore().reviewAutoAdvance = false
    const page = await mountAt('/review/s1')

    await page.get('[data-testid="review-accept"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.key).toBeUndefined()
    expect(page.findAll('[data-testid="review-claim-body"]')).toHaveLength(0)
    expect(page.findAll('[data-testid="review-item"]')).toHaveLength(2)
    await page.findAll('[data-testid="review-item-title"]')[0]!.trigger('click')
    await flushPromises()
    expect(page.findAll('[data-testid="review-claim-body"]')).toHaveLength(1)
    expect(store.acceptStep).toHaveBeenCalledOnce()
  })

  it('asks before sending a claim back when Settings says so', async () => {
    const store = seed()
    store.sendBackStep = vi.fn().mockResolvedValue(undefined)
    usePreferencesStore().reviewConfirmSendBack = true
    const page = await mountAt('/review/s2')

    await page.findAll('[data-testid="review-send-back"]')[1]!.trigger('click')
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

  describe('keys', () => {
    it('moves through the claims with J and K and stops at either end', async () => {
      seed()
      await mountAt('/review/s1')

      press('j')
      await flushPromises()
      expect(router.currentRoute.value.params.key).toBe('s2')
      press('j')
      press('j')
      await flushPromises()
      expect(router.currentRoute.value.params.key).toBe('rk-3')
      press('k')
      await flushPromises()
      expect(router.currentRoute.value.params.key).toBe('s2')
    })

    it('accepts the open claim with A', async () => {
      const store = seed()
      await mountAt('/review/s2')

      press('a')
      await flushPromises()

      expect(store.acceptStep).toHaveBeenCalledWith('s2')
    })

    it('sends the open claim back with S', async () => {
      const store = seed()
      await mountAt('/review/s2')

      press('s')
      await flushPromises()

      expect(store.sendBackStep).toHaveBeenCalledWith('s2')
    })

    it('ignores the keys while a field has focus', async () => {
      const store = seed()
      await mountAt('/review/s1')
      const field = document.createElement('input')
      document.body.appendChild(field)

      press('a', field)
      await flushPromises()

      expect(store.acceptStep).not.toHaveBeenCalled()
    })

    it('ignores the keys while the send-back question is open', async () => {
      const store = seed()
      usePreferencesStore().reviewConfirmSendBack = true
      await mountAt('/review/s1')

      press('s')
      await flushPromises()
      press('a')
      await flushPromises()

      expect(useSendBackGuardStore().pending).not.toBeNull()
      expect(store.acceptStep).not.toHaveBeenCalled()
    })
  })
})
