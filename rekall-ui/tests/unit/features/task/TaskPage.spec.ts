import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import TaskPage from '@/features/task/components/TaskPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/features/adherence/api/adherence.api', () => ({
  fetchTaskAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, deviationSteps: [], visits: []
  })),
  fetchProjectAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, mostSentBack: []
  }))
}))

const stub = { template: '<div />' }

function makeTask(id: string, overrides: Partial<Task> = {}): Task {
  return {
    id: id as TaskId,
    code: id.toUpperCase(), label: id,
    title: `Task ${id}`,
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'Acme',
    projectRepoFolder: '/work/vega',
    documentCount: 2,
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
    graphLaidAt: '2026-10-01T08:00:00Z',
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
    anchor: `task:${id.toUpperCase()}`,
    updatedAt: '2026-10-01T10:00:00Z',
    ...overrides
  }
}

function claimedStep(taskId: string): TaskStep {
  return {
    id: 's1' as TaskStepId, taskId: taskId as TaskId, title: 'Wire the form', bodyMarkdown: null, state: 'CLAIMED', done: false,
    runningAt: null, claimedAt: '2026-10-01T09:00:00Z', doneAt: null, position: 0, flowX: null, flowY: null, maxVisits: null,
    startsSession: false, connectionId: null, passes: [], links: [], visits: [],
    createdAt: '2026-10-01T07:00:00Z', updatedAt: '2026-10-01T07:00:00Z'
  }
}

let pinia: Pinia
let router: Router

function seed() {
  const store = useConsoleStore()
  store.tasks = [makeTask('rk-1'), makeTask('rk-2')]
  store.steps = [claimedStep('rk-1')]
  store.isLoading = false
  store.setTaskStatus = vi.fn().mockResolvedValue(undefined)
  return store
}

async function mountAt(path: string) {
  await router.push(path)
  await router.isReady()
  const wrapper = mount(TaskPage, {
    attachTo: document.body,
    global: {
      plugins: [pinia, router],
      stubs: {
        OpenTerminalButton: true,
        LaunchClaudeCodeButton: true,
        TaskPlanTab: { template: '<div data-testid="plan-tab-stub" />' },
        TaskNotesTab: { template: '<div data-testid="notes-tab-stub" />' },
        TaskDescriptionTab: { template: '<div data-testid="description-tab-stub" />' },
        TaskTimeTab: { template: '<div data-testid="time-tab-stub" />' },
        TaskDeviationsTab: { template: '<div data-testid="deviations-tab-stub" />' },
        WrapupPane: { template: '<div data-testid="wrapup-pane-stub" />' },
        TaskTokenChip: true
      }
    }
  })
  await flushPromises()
  return wrapper
}

describe('TaskPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'work', component: stub },
        { path: '/task/:id/:tab?', name: 'task', component: TaskPage },
        { path: '/console', name: 'console', component: stub },
        { path: '/projects/:id', name: 'project-detail', component: stub }
      ]
    })
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('selects the task the route names and says where it sits and what it is', async () => {
    const store = seed()
    const page = await mountAt('/task/rk-1/plan')

    expect(store.selectedTaskId).toBe('rk-1')
    expect(page.get('[data-testid="task-title"]').text()).toBe('Task rk-1')
    expect(page.get('[data-testid="crumb-project"]').text()).toBe('Acme · Vega')
    expect(page.get('[data-testid="task-anchor"]').text()).toBe('/rk task:RK-1')
    expect(page.get('[data-testid="task-status"]').text()).toContain('Awaiting review')
  })

  it('lists the six tabs as links, with Plan counting its steps and turning amber while one waits', async () => {
    seed()
    const page = await mountAt('/task/rk-1/plan')

    const tabs = page.findAll('[role="tab"]')
    expect(tabs.map((tab) => tab.text().replace(/\s*\d+$/, ''))).toEqual(['Plan', 'Wrapup', 'Notes', 'Description', 'Time', 'Deviations'])
    expect(tabs[0]!.get('[data-testid="sub-nav-count"]').text()).toBe('1')
    expect(tabs[2]!.attributes('href')).toBe('/task/rk-1/notes')
    expect(tabs[2]!.get('[data-testid="sub-nav-count"]').text()).toBe('2')
  })

  it('shows the pane each tab names', async () => {
    seed()
    const shown: Record<string, string> = {
      plan: 'plan-tab-stub',
      wrapup: 'wrapup-pane-stub',
      notes: 'notes-tab-stub',
      description: 'description-tab-stub',
      time: 'time-tab-stub',
      deviations: 'deviations-tab-stub'
    }

    for (const [tab, testid] of Object.entries(shown)) {
      const page = await mountAt(`/task/rk-1/${tab}`)
      expect(page.find(`[data-testid="${testid}"]`).exists()).toBe(true)
      page.unmount()
    }
  })

  it('opens an epic on its description, which has no plan', async () => {
    const store = seed()
    store.tasks = [makeTask('rk-1', { kind: 'EPIC' })]
    const page = await mountAt('/task/rk-1/plan')

    expect(router.currentRoute.value.fullPath).toBe('/task/rk-1/description')
    expect(page.findAll('[role="tab"]').map((tab) => tab.text())).not.toContain('Plan')
  })

  it('follows a pane opened elsewhere to the tab that shows it', async () => {
    const store = seed()
    await mountAt('/task/rk-1/plan')

    store.openWrapup()
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/rk-1/wrapup')
  })

  it('says so when the task does not exist', async () => {
    seed()
    const page = await mountAt('/task/nope/plan')

    expect(page.get('[data-testid="task-missing"]').text()).toContain('No such task.')
  })

  it('follows the route to another task', async () => {
    const store = seed()
    const page = await mountAt('/task/rk-1/plan')

    await router.push('/task/rk-2/plan')
    await flushPromises()

    expect(store.selectedTaskId).toBe('rk-2')
    expect(page.get('[data-testid="task-title"]').text()).toBe('Task rk-2')
  })

  it('follows a task selected elsewhere, as search does, by moving to its page', async () => {
    const store = seed()
    await mountAt('/task/rk-1/plan')

    store.selectTask('rk-2' as TaskId)
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/rk-2/plan')
  })

  it('changes the status from the pill and confirms it', async () => {
    const store = seed()
    const page = await mountAt('/task/rk-1/plan')

    await page.get('[data-testid="task-status"]').trigger('click')
    const items = page.findAll('[data-testid="popover-menu-item"]')
    await items.find((item) => item.text() === 'Blocked')!.trigger('click')
    await flushPromises()

    expect(store.setTaskStatus).toHaveBeenCalledWith('rk-1', 'BLOCKED')
    expect(useToastStore().toasts.some((toast) => toast.message === 'Status set to blocked')).toBe(true)
  })
})
