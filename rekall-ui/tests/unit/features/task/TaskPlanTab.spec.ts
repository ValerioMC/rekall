import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import type * as VueRouter from 'vue-router'
import TaskPlanTab from '@/features/task/components/TaskPlanTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { CommitReference } from '@/features/commits/model/commitReference'
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

  it('starts on the overview, which leads with the claim waiting, and opens it on Review next', async () => {
    const store = seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'CLAIMED'), makeStep('s3', 2, 'OPEN')])
    const tab = render()

    expect(tab.find('[data-testid="decision-panel"]').exists()).toBe(false)
    expect(tab.get('[data-testid="overview-title"]').text()).toBe('1 step waits for you')
    expect(tab.get('[data-testid="queue-waiting"]').text()).toContain('Step s2')

    await tab.get('[data-testid="overview-review-next"]').trigger('click')
    const panel = tab.get('[data-testid="decision-panel"]')
    expect(panel.get('[data-testid="decision-state"]').text()).toBe('Claimed')
    expect(panel.get('[data-testid="decision-position"]').text()).toBe('Step 2 of 3')

    await panel.get('[data-testid="step-accept"]').trigger('click')
    await flushPromises()

    expect(store.acceptStep).toHaveBeenCalledWith('s2')
    expect(useToastStore().toasts.some((toast) => toast.action?.label === 'Reopen')).toBe(true)
  })

  it('sends a claim back from the decision panel', async () => {
    const store = seed([makeStep('s1', 0, 'CLAIMED')])
    const tab = render()
    await tab.get('[data-testid="overview-review-next"]').trigger('click')

    await tab.get('[data-testid="step-send-back"]').trigger('click')
    await flushPromises()

    expect(store.sendBackStep).toHaveBeenCalledWith('s1')
  })

  it('shows the pressed step in the panel', async () => {
    seed([makeStep('s1', 0, 'CLAIMED'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'CLAIMED')])
    const tab = render()

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
    await tab.get('[data-testid="plan-step-title"]').trigger('click')

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

  describe('drafts', () => {
    function draft(id: string, position: number, overrides: Partial<TaskStep> = {}): TaskStep {
      return { ...makeStep(id, position, 'DRAFT'), ...overrides }
    }

    it('opens a draft in the pane beside the list and keeps its row a line', async () => {
      seed([makeStep('s1', 0, 'OPEN'), draft('s2', 1)])
      const tab = render()

      await tab.get('[data-testid="draft-title"]').trigger('click')

      expect(tab.get('[data-testid="draft-title-field"]').element).toHaveProperty('value', 'Step s2')
      expect(tab.get('[data-testid="draft-panel"]').text()).toContain('Draft 1 of 1')
      expect(tab.get('[data-testid="drafts"]').find('input').exists()).toBe(false)
    })

    it('waits for a detail before a draft can be promoted, in the row and in the pane', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), draft('s2', 1), draft('s3', 2, { bodyMarkdown: 'Write the migration.' })])
      const tab = render()
      const [empty, written] = tab.findAll('[data-testid="draft-row"]')

      expect(empty!.get('[data-testid="draft-promote"]').attributes('disabled')).toBeDefined()
      expect(written!.get('[data-testid="draft-promote"]').attributes('disabled')).toBeUndefined()

      await empty!.get('[data-testid="draft-title"]').trigger('click')
      expect(tab.get('[data-testid="draft-panel"] [data-testid="draft-promote"]').attributes('disabled')).toBeDefined()
      expect(tab.get('[data-testid="draft-promote-hint"]').text()).toBe('Write the detail to promote it.')

      await written!.get('[data-testid="draft-title"]').trigger('click')
      await tab.get('[data-testid="draft-panel"] [data-testid="draft-promote"]').trigger('click')
      await flushPromises()
      expect(store.promoteStep).toHaveBeenCalledWith('s3')
    })

    it('chooses the session of a draft in the pane', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), draft('s2', 1, { bodyMarkdown: 'Detail' })])
      const tab = render()
      await tab.get('[data-testid="draft-title"]').trigger('click')

      await tab.get('[data-testid="step-session"] [data-testid="segment-new"]').trigger('click')
      await flushPromises()

      expect(store.beginStepSession).toHaveBeenCalledWith('s2', true)
    })

    it('shows the routes of a flow draft in the pane and counts them on its row', async () => {
      const links = [
        { id: 'l1', fromStepId: 's2', toStepId: 's1', outcome: 'pass', position: 0 },
        { id: 'l2', fromStepId: 's2', toStepId: null, outcome: 'fail', position: 1 }
      ] as unknown as TaskStep['links']
      seed([makeStep('s1', 0, 'OPEN'), draft('s2', 1, { links })])
      const tab = render()

      expect(tab.get('[data-testid="draft-routes"]').text()).toBe('2 routes')
      await tab.get('[data-testid="draft-title"]').trigger('click')

      const routes = tab.get('[data-testid="step-routes"]').text()
      expect(routes).toContain('pass')
      expect(routes).toContain('Step s1')
      expect(routes).toContain('end')
    })

    it('opens a draft just added in the pane', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN')])
      store.addStep = vi.fn().mockImplementation(async (_task: TaskId, title: string) => {
        const created = { ...draft('s9', 1), title }
        store.steps = [...store.steps, created]
        return created
      })
      const tab = render()

      await tab.get('[data-testid="new-step"]').setValue('Write the migration')
      await tab.get('form').trigger('submit')
      await flushPromises()

      expect(tab.get('[data-testid="draft-title-field"]').element).toHaveProperty('value', 'Write the migration')
    })

    it('opens the first draft of a task that has nothing but drafts', async () => {
      seed([draft('s1', 0), draft('s2', 1)])
      const tab = render()
      await flushPromises()

      expect(tab.get('[data-testid="draft-title-field"]').element).toHaveProperty('value', 'Step s1')
      expect(tab.find('[data-testid="draft-overview"]').exists()).toBe(false)
    })
  })

  it('hides the step list while a step is in focus and brings it back', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'CLAIMED')])
    const tab = render()
    await tab.get('[data-testid="overview-review-next"]').trigger('click')
    const list = tab.get('section[aria-label="Steps"]')
    expect(list.classes()).not.toContain('hidden')

    await tab.get('[data-testid="decision-focus"]').trigger('click')
    expect(list.classes()).toContain('hidden')
    expect(tab.get('[data-testid="decision-focus"]').attributes('aria-pressed')).toBe('true')

    await tab.get('[data-testid="decision-focus"]').trigger('click')
    expect(list.classes()).not.toContain('hidden')
  })

  it('steps through the list with J and K and the arrows in the panel header', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'OPEN')])
    const tab = render()
    await tab.findAll('[data-testid="plan-step-title"]')[1]!.trigger('click')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'j' }))
    await flushPromises()
    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s3')
    expect(tab.get('[data-testid="decision-next"]').attributes('disabled')).toBeDefined()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k' }))
    await flushPromises()
    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s2')

    await tab.get('[data-testid="decision-previous"]').trigger('click')
    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s1')
  })

  it('leaves J and K to a field being typed in', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN')])
    const tab = render()
    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')

    tab.get('[data-testid="new-step"]').element.dispatchEvent(new KeyboardEvent('keydown', { key: 'j', bubbles: true }))
    await flushPromises()

    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s1')
  })

  it('keeps the list and the pane in place whether or not a step is selected', async () => {
    seed([makeStep('s1', 0, 'DONE'), makeStep('s2', 1, 'CLAIMED')])
    const tab = render()
    const list = tab.get('section[aria-label="Steps"]')
    const resting = list.classes().join(' ')

    expect(tab.find('[data-testid="plan-overview"]').exists()).toBe(true)
    await tab.findAll('[data-testid="plan-step-title"]')[1]!.trigger('click')

    expect(tab.find('[data-testid="plan-overview"]').exists()).toBe(false)
    expect(tab.find('[data-testid="decision-panel"]').exists()).toBe(true)
    expect(tab.get('section[aria-label="Steps"]').classes().join(' ')).toBe(resting)
  })

  it('goes back to the overview on Esc and from the Overview key in the panel header', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN')])
    const tab = render()
    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    expect(tab.find('[data-testid="plan-overview"]').exists()).toBe(true)

    await tab.findAll('[data-testid="plan-step-title"]')[1]!.trigger('click')
    await tab.get('[data-testid="decision-overview"]').trigger('click')
    expect(tab.find('[data-testid="plan-overview"]').exists()).toBe(true)
  })

  it('opens the first step on J while none is selected', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN')])
    const tab = render()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'j' }))
    await flushPromises()

    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s1')
  })

  it('keeps the progress and the keys in one row that stays whatever is selected', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN')])
    const tab = render()

    expect(tab.get('[data-testid="plan-bar"]').find('[data-testid="plan-tube"]').exists()).toBe(true)

    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')

    expect(tab.get('[data-testid="plan-bar"]').find('[data-testid="plan-tube"]').exists()).toBe(true)
  })

  function commitOn(stepId: string): CommitReference {
    return {
      id: 'c1', taskId, stepId: stepId as TaskStepId, stepTitle: null, componentId: null, componentName: null,
      commitHash: 'a3f9c21', comment: 'expose the import', inContext: false, createdAt: '2026-10-01T09:30:00Z'
    }
  }

  it('opens a claim on its evidence and folds it away from the commits chip', async () => {
    const store = seed([makeStep('s1', 0, 'CLAIMED')])
    store.commitReferences = [commitOn('s1')]
    const tab = render()
    await tab.get('[data-testid="overview-review-next"]').trigger('click')

    expect(tab.find('[data-testid="step-evidence"]').exists()).toBe(true)

    await tab.get('[data-testid="decision-commits"]').trigger('click')
    expect(tab.find('[data-testid="step-evidence"]').exists()).toBe(false)

    await tab.get('[data-testid="decision-commits"]').trigger('click')
    expect(tab.find('[data-testid="step-evidence"]').exists()).toBe(true)
  })

  it('keeps the evidence of a step that was accepted behind its chip until it is asked for', async () => {
    const store = seed([makeStep('s1', 0, 'DONE')])
    store.commitReferences = [commitOn('s1')]
    const tab = render()
    await tab.get('[data-testid="plan-step-title"]').trigger('click')

    expect(tab.find('[data-testid="step-evidence"]').exists()).toBe(false)
    await tab.get('[data-testid="decision-commits"]').trigger('click')
    expect(tab.find('[data-testid="step-evidence"]').exists()).toBe(true)
  })

  it('reopens an accepted step from the header, which asks twice, and has no dock', async () => {
    const store = seed([makeStep('s1', 0, 'DONE')])
    const tab = render()
    await tab.get('[data-testid="plan-step-title"]').trigger('click')

    expect(tab.find('[data-testid="decision-actions"]').exists()).toBe(false)
    await tab.get('[data-testid="step-reopen"]').trigger('click')
    expect(store.reopenStep).not.toHaveBeenCalled()
    expect(tab.get('[data-testid="step-reopen"]').text()).toBe('Confirm reopen')

    await tab.get('[data-testid="step-reopen"]').trigger('click')
    await flushPromises()
    expect(store.reopenStep).toHaveBeenCalledWith('s1')
  })

  it('shows the dock only while a decision is still to be made', async () => {
    seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'DONE')])
    const tab = render()

    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')
    expect(tab.get('[data-testid="decision-actions"]').find('[data-testid="decision-mark-done"]').exists()).toBe(true)

    await tab.findAll('[data-testid="plan-step-title"]')[1]!.trigger('click')
    expect(tab.find('[data-testid="decision-actions"]').exists()).toBe(false)
  })

  it('renames the step in place from its heading and saves what was typed', async () => {
    const store = seed([makeStep('s1', 0, 'OPEN')])
    const tab = render()
    await tab.get('[data-testid="plan-step-title"]').trigger('click')

    await tab.get('[data-testid="decision-title"]').trigger('click')
    await tab.get('[data-testid="step-title-field"]').setValue('A better name')
    await tab.get('[data-testid="step-title-field"]').trigger('keydown.enter')
    await new Promise((resolve) => setTimeout(resolve, 800))

    expect(store.saveStep).toHaveBeenCalledWith('s1', { title: 'A better name', bodyMarkdown: '' })
    expect(tab.find('[data-testid="step-title-field"]').exists()).toBe(false)
  })

  it('keeps the old title when the new one is emptied', async () => {
    seed([makeStep('s1', 0, 'OPEN')])
    const tab = render()
    await tab.get('[data-testid="plan-step-title"]').trigger('click')
    await tab.get('[data-testid="decision-title"]').trigger('click')

    await tab.get('[data-testid="step-title-field"]').setValue('  ')
    await tab.get('[data-testid="step-title-field"]').trigger('keydown.enter')

    expect(tab.get('[data-testid="decision-title"]').text()).toBe('Step s1')
  })

  it('holds a step to the page width the reader picked, and offers no second width menu beside it', async () => {
    seed([makeStep('s1', 0, 'OPEN')])
    const tab = render()
    await tab.findAll('[data-testid="plan-step-title"]')[0]!.trigger('click')

    expect(tab.findAll('[data-testid="task-width"]')).toHaveLength(1)
    expect(tab.get('[data-testid="decision-panel"] header > div').attributes('style')).toContain('max-width: 1360px')

    await tab.get('[data-testid="task-width"]').trigger('click')
    await tab.findAll('[data-testid="popover-menu-item"]').find((item) => item.text() === 'Full')!.trigger('click')

    expect(tab.get('[data-testid="decision-panel"] header > div').attributes('style')).toContain('max-width: none')
  })

  it('invites the first step on a task with none', () => {
    seed([])
    const tab = render()

    expect(tab.find('[data-testid="write-first-step"]').exists()).toBe(true)
    expect(tab.find('[data-testid="decision-panel"]').exists()).toBe(false)
    expect(tab.find('[data-testid="plan-bar"]').exists()).toBe(false)
  })
  describe('sessions', () => {
    function opening(step: TaskStep): TaskStep {
      return { ...step, startsSession: true }
    }

    function menuItem(tab: ReturnType<typeof render>, label: string) {
      return tab.findAll('[data-testid="popover-menu-item"]').find((item) => item.text().includes(label))!
    }

    it('groups the steps under the session they run in', () => {
      seed([makeStep('s1', 0, 'DONE'), opening(makeStep('s2', 1, 'CLAIMED')), makeStep('s3', 2, 'OPEN')])
      const tab = render()

      expect(tab.findAll('[data-testid="plan-session"]').map((session) => session.attributes('data-standing'))).toEqual(['accepted', 'waiting'])
      expect(tab.findAll('[data-testid="session-note"]')[1]!.text()).toBe('Waiting for you · 1 claimed')
    })

    it('cuts a session from the seam between two open steps', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'OPEN')])
      const tab = render()

      await tab.findAll('[data-testid="step-new-session"]')[0]!.trigger('click')

      expect(store.beginStepSession).toHaveBeenCalledWith('s2', true)
    })

    it('folds a session that has not started into the one above from its header', async () => {
      const store = seed([makeStep('s1', 0, 'DONE'), opening(makeStep('s2', 1, 'OPEN'))])
      const tab = render()

      await tab.get('[data-testid="session-join"]').trigger('click')

      expect(store.beginStepSession).toHaveBeenCalledWith('s2', false)
    })

    it('moves an open step with Alt and the arrow keys, handing the session opening over with it', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), opening(makeStep('s2', 1, 'OPEN')), makeStep('s3', 2, 'OPEN')])
      const tab = render()

      await tab.findAll('[data-testid="plan-step"]')[2]!.trigger('keydown', { key: 'ArrowUp', altKey: true })
      await flushPromises()

      expect(store.moveStep).toHaveBeenCalledWith('s3', 1)
      expect(store.beginStepSession).toHaveBeenCalledWith('s3', true)
      expect(store.beginStepSession).toHaveBeenCalledWith('s2', false)
    })

    it('does not move a step past one that has been started', async () => {
      const store = seed([makeStep('s1', 0, 'CLAIMED'), makeStep('s2', 1, 'OPEN')])
      const tab = render()

      await tab.findAll('[data-testid="plan-step"]')[1]!.trigger('keydown', { key: 'ArrowUp', altKey: true })
      await flushPromises()

      expect(store.moveStep).not.toHaveBeenCalled()
    })

    it('returns a step to draft from its menu and puts it back, in its place, on Undo', async () => {
      const store = seed([makeStep('s1', 0, 'DONE'), opening(makeStep('s2', 1, 'OPEN')), makeStep('s3', 2, 'OPEN')])
      store.returnStepToDraft = vi.fn().mockResolvedValue(undefined)
      const tab = render()

      await tab.findAll('[data-testid="step-more"]')[0]!.trigger('click')
      await menuItem(tab, 'Return to draft').trigger('click')
      await flushPromises()

      expect(store.returnStepToDraft).toHaveBeenCalledWith('s2')
      expect(store.beginStepSession).toHaveBeenCalledWith('s3', true)
      const toast = useToastStore().toasts[0]!
      expect(toast.message).toBe('Step s2 is a draft now')

      toast.action!.run()
      await flushPromises()

      expect(store.promoteStep).toHaveBeenCalledWith('s2')
      expect(store.moveStep).toHaveBeenCalledWith('s2', 1)
      expect(store.beginStepSession).toHaveBeenCalledWith('s3', false)
    })

    function drag(tab: ReturnType<typeof render>, selector: string, index: number) {
      const handle = tab.findAll(selector)[index]!
      return handle.trigger('pointerdown').then(() => {
        const row = handle.element.closest('[draggable]') as HTMLElement
        row.dispatchEvent(Object.assign(new Event('dragstart', { bubbles: true, cancelable: true }), { dataTransfer: { setData: vi.fn() } }))
      })
    }

    async function dropOnSlot(tab: ReturnType<typeof render>, index: number): Promise<void> {
      const slot = tab.findAll('[data-testid="plan-session-slot"]')[index]!
      await slot.trigger('dragover')
      await slot.trigger('drop')
      await flushPromises()
    }

    it('runs a step in a session of its own when it is dropped on the slot after the last session, and puts it back on Undo', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), makeStep('s2', 1, 'OPEN'), makeStep('s3', 2, 'OPEN')])
      const tab = render()

      await drag(tab, '[data-testid="step-handle"]', 0)
      const slots = tab.findAll('[data-testid="plan-session-slot"]')
      expect(slots.map((slot) => slot.text())).toEqual(['New session here', 'New session at the end'])
      await dropOnSlot(tab, 1)

      expect(store.moveStep).toHaveBeenCalledWith('s1', 2)
      expect(store.beginStepSession).toHaveBeenCalledWith('s1', true)
      expect(store.beginStepSession).toHaveBeenCalledTimes(1)
      const toast = useToastStore().toasts[0]!
      expect(toast.message).toBe('Step s1 runs in a session of its own')
      expect(toast.action?.label).toBe('Undo')
    })

    it('offers no slot while nothing is held', () => {
      seed([makeStep('s1', 0, 'OPEN'), opening(makeStep('s2', 1, 'OPEN'))])

      expect(render().find('[data-testid="plan-session-slot"]').exists()).toBe(false)
    })

    it('opens a slot in front of every session that has not started, and none in front of one that has', async () => {
      seed([makeStep('s1', 0, 'CLAIMED'), opening(makeStep('s2', 1, 'OPEN')), opening(makeStep('s3', 2, 'OPEN'))])
      const tab = render()

      await drag(tab, '[data-testid="step-handle"]', 1)

      expect(tab.findAll('[data-testid="plan-session-slot"]').map((slot) => slot.text())).toEqual([
        'New session here',
        'New session here',
        'New session at the end'
      ])
    })

    it('runs a step in the session chosen from its menu, and says it cannot join one that has started', async () => {
      const store = seed([makeStep('s1', 0, 'CLAIMED'), opening(makeStep('s2', 1, 'OPEN')), opening(makeStep('s3', 2, 'OPEN'))])
      const tab = render()

      await tab.findAll('[data-testid="step-more"]')[0]!.trigger('click')
      expect(menuItem(tab, 'Session 1').attributes('disabled')).toBeDefined()
      await menuItem(tab, 'Session 3').trigger('click')
      await flushPromises()

      expect(store.moveStep).not.toHaveBeenCalled()
      expect(store.beginStepSession).toHaveBeenCalledWith('s3', false)
    })

    it('moves a whole session by its grip and lays the sessions out again on Undo', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN'), opening(makeStep('s2', 1, 'OPEN')), makeStep('s3', 2, 'OPEN')])
      const tab = render()

      await drag(tab, '[data-testid="session-handle"]', 1)
      expect(tab.findAll('[data-testid="plan-session-slot"]').map((slot) => slot.text())).toEqual(['New session here'])
      await dropOnSlot(tab, 0)

      expect(store.moveStep).toHaveBeenCalledWith('s1', 2)
      expect(useToastStore().toasts[0]!.message).toBe('Moved Session 2')
    })

    it('promotes a draft where it is dropped, in a session of its own', async () => {
      const store = seed([
        makeStep('s1', 0, 'OPEN'),
        { ...makeStep('d1', 1, 'DRAFT'), bodyMarkdown: 'Write the toast.' }
      ])
      store.promoteStep = vi.fn().mockImplementation(async (id: string) => {
        store.steps = store.steps.map((step) => (step.id === id ? { ...step, state: 'OPEN' as const } : step))
      })
      const tab = render()

      await drag(tab, '[data-testid="draft-handle"]', 0)
      await dropOnSlot(tab, 1)

      expect(store.promoteStep).toHaveBeenCalledWith('d1')
      expect(store.beginStepSession).toHaveBeenCalledWith('d1', true)
      expect(useToastStore().toasts[0]!.message).toBe('Promoted Step d1')
    })

    it('promotes a draft into the session picked beside its Promote key', async () => {
      const store = seed([
        makeStep('s1', 0, 'OPEN'),
        opening(makeStep('s2', 1, 'OPEN')),
        { ...makeStep('d1', 2, 'DRAFT'), bodyMarkdown: 'Write the toast.' }
      ])
      store.promoteStep = vi.fn().mockImplementation(async (id: string) => {
        store.steps = store.steps.map((step) => (step.id === id ? { ...step, state: 'OPEN' as const } : step))
      })
      const tab = render()

      await tab.get('[data-testid="draft-promote-into"]').trigger('click')
      await menuItem(tab, 'Session 1').trigger('click')
      await flushPromises()

      expect(store.promoteStep).toHaveBeenCalledWith('d1')
      expect(store.moveStep).toHaveBeenCalledWith('d1', 1)
      expect(store.beginStepSession).not.toHaveBeenCalled()
    })

    it('offers to undo ticking an open step from its seal', async () => {
      const store = seed([makeStep('s1', 0, 'OPEN')])
      const tab = render()

      await tab.get('[data-testid="plan-step-seal"]').trigger('click')
      await flushPromises()

      expect(store.toggleStep).toHaveBeenCalledWith('s1')
      const toast = useToastStore().toasts[0]!
      expect(toast.message).toBe('Marked Step s1 done')
      expect(toast.action?.label).toBe('Undo')

      toast.action!.run()
      await flushPromises()

      expect(store.reopenStep).toHaveBeenCalledWith('s1')
    })
  })
})
