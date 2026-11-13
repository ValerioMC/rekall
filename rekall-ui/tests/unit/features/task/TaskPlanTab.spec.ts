import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import type * as VueRouter from 'vue-router'
import TaskPlanTab from '@/features/task/components/TaskPlanTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

const push = vi.fn()

vi.mock('vue-router', async (importOriginal) => ({
  ...(await importOriginal<typeof VueRouter>()),
  useRouter: () => ({ push })
}))
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

const taskId = 't1' as TaskId

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: taskId,
    code: 'RK-1', label: 'report-builder',
    title: 'Report builder',
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'Acme',
    projectRepoFolder: '/work/vega',
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
    graphLaidAt: '2026-10-01T08:00:00Z',
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
    anchor: 'task:RK-1',
    updatedAt: '2026-10-01T10:00:00Z',
    ...overrides
  }
}

function makeStep(id: string, position: number, state: TaskStepState): TaskStep {
  return {
    id: id as TaskStepId,
    taskId,
    title: `Step ${id}`,
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: state === 'CLAIMED' ? '2026-10-01T09:00:00Z' : null,
    doneAt: state === 'DONE' ? '2026-10-01T08:00:00Z' : null,
    position,
    flowX: null,
    flowY: null,
    maxVisits: null,
    startsSession: false,
    connectionId: null,
    passes: [],
    links: [],
    visits: [],
    createdAt: '2026-10-01T07:00:00Z',
    updatedAt: '2026-10-01T07:00:00Z'
  }
}

let pinia: Pinia

function seed(steps: TaskStep[], task: Task = makeTask()) {
  const store = useConsoleStore()
  store.tasks = [task]
  store.selectedTaskId = taskId
  store.steps = steps
  store.isLoading = false
  store.acceptStep = vi.fn().mockResolvedValue(undefined)
  store.acceptTask = vi.fn().mockResolvedValue(undefined)
  store.sendBackTask = vi.fn().mockResolvedValue(undefined)
  store.toggleStep = vi.fn().mockResolvedValue(undefined)
  store.sendBackStep = vi.fn().mockResolvedValue(undefined)
  store.reopenStep = vi.fn().mockResolvedValue(undefined)
  store.promoteStep = vi.fn().mockResolvedValue(undefined)
  store.removeStep = vi.fn().mockResolvedValue(undefined)
  store.moveStep = vi.fn().mockResolvedValue(undefined)
  store.beginStepSession = vi.fn().mockResolvedValue(undefined)
  store.saveStep = vi.fn().mockResolvedValue(undefined)
  store.addStep = vi.fn().mockResolvedValue(null)
  return store
}

function render() {
  return mount(TaskPlanTab, {
    props: { task: useConsoleStore().tasks[0]! },
    attachTo: document.body,
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        OpenTerminalButton: true,
        LaunchClaudeCodeButton: true,
        LogCommitButton: true,
        CommitReferenceList: true,
        TaskBranchPanel: true,
        GraphAttachPanel: true,
        StepFlowView: true,
        AppConfirm: {
          template: '<div data-testid="confirm"><button data-testid="confirm-yes" @click="$emit(\'confirm\')" /></div>'
        }
      }
    }
  })
}

describe('TaskPlanTab', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    push.mockClear()
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('lists the checklist as numbered rows and keeps the drafts under their own heading', () => {
    seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'DRAFT')])
    const tab = render()

    const rows = tab.findAll('[data-testid="plan-step"]')
    expect(rows.map((row) => row.get('[data-testid="plan-step-number"]').text())).toEqual(['1', '2'])
    expect(tab.get('[data-testid="drafts"]').text()).toContain('Step s3')
  })

  it('opens the claim waiting for a decision and accepts it from the decision panel', async () => {
    const store = seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'CLAIMED'), makeStep('s3', 2, 'OPEN')])
    const tab = render()

    const panel = tab.get('[data-testid="decision-panel"]')
    expect(panel.get('[data-testid="decision-kicker"]').text()).toContain('Step 2 of 3 · claimed')

    await panel.get('[data-testid="step-accept"]').trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s2')
    expect(useToastStore().toasts.some((toast) => toast.action?.label === 'Reopen')).toBe(true)
  })

  it('sends a claim back from the decision panel', async () => {
    const store = seed([makeStep('s1', 0, 'CLAIMED')])
    const tab = render()

    await tab.get('[data-testid="step-send-back"]').trigger('click')
    await flushPromises()

    expect(store.sendBackStep).toHaveBeenCalledWith('s1')
  })

  it('shows the pressed step in the panel and lists the other claims as up next', async () => {
    seed([makeStep('s1', 0, 'CLAIMED'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'CLAIMED')])
    const tab = render()

    expect(tab.get('[data-testid="decision-up-next"]').text()).toContain('Step s3')

    await tab.findAll('[data-testid="plan-step-title"]')[1]!.trigger('click')

    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s2')
    expect(tab.get('[data-testid="decision-panel"]').attributes('data-step-state')).toBe('OPEN')
  })

  it('accepts a claim from the seal on its row', async () => {
    const store = seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'CLAIMED')])
    const tab = render()

    await tab.findAll('[data-testid="plan-step-seal"]')[1]!.trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s2')
  })

  it('ticks an open step from the panel: marking it done is a person’s move', async () => {
    const store = seed([makeStep('s1', 0, 'OPEN')])
    const tab = render()

    await tab.get('[data-testid="decision-mark-done"]').trigger('click')
    await flushPromises()

    expect(store.toggleStep).toHaveBeenCalledWith('s1')
  })

  it('moves the panel to the first claim on Review next', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'CLAIMED')])
    const tab = render()
    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')
    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s1')

    await tab.get('[data-testid="plan-review-next"]').trigger('click')

    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s2')
  })

  it('hides the done steps and shows them again', async () => {
    seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'OPEN')])
    const tab = render()

    await tab.get('[data-testid="toggle-hide-done"]').trigger('click')
    expect(tab.findAll('[data-testid="plan-step"]')).toHaveLength(1)

    await tab.get('[data-testid="toggle-hide-done"]').trigger('click')
    expect(tab.findAll('[data-testid="plan-step"]')).toHaveLength(2)
  })

  it('offers to accept everything once the claimed steps are all that is left', async () => {
    const store = seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'CLAIMED')])
    const tab = render()

    await tab.get('[data-testid="steps-accept-task"]').trigger('click')
    await flushPromises()

    expect(store.acceptTask).toHaveBeenCalledWith(taskId)
  })

  it('reviews a task handed in whole, which has no steps to claim', async () => {
    const store = seed([], makeTask({ reviewActive: true, reviewState: 'CLAIMED', claimedAt: '2026-10-01T09:00:00Z' }))
    const tab = render()

    expect(tab.get('[data-testid="plan-handed-in"]').text()).toContain('handed in whole')
    await tab.get('[data-testid="handed-in-accept"]').trigger('click')
    await flushPromises()

    expect(store.acceptTask).toHaveBeenCalledWith(taskId)
  })

  it('adds a draft step from the line under the list', async () => {
    const store = seed([makeStep('s1', 0, 'OPEN')])
    store.addStep = vi.fn().mockResolvedValue(makeStep('s9', 1, 'DRAFT'))
    const tab = render()

    await tab.get('[data-testid="new-step"]').setValue('Write the migration')
    await tab.get('form').trigger('submit')
    await flushPromises()

    expect(store.addStep).toHaveBeenCalledWith(taskId, 'Write the migration')
  })

  it('invites the first step on a task with none', () => {
    seed([])
    const tab = render()

    expect(tab.find('[data-testid="write-first-step"]').exists()).toBe(true)
    expect(tab.find('[data-testid="decision-panel"]').exists()).toBe(false)
  })
})
