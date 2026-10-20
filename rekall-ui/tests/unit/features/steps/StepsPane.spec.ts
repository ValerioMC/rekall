import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import StepsPane from '@/features/steps/components/StepsPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import { STEP_DETAIL_TEMPLATE } from '@/model/templates'
import type { Task, TaskStep, TaskStepState } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

/**
 * The checklist pane as a person drives it: the ledger and its counts, the step that opens on
 * its own, ticking, accepting, reopening, the drafts shelf, editing a step's detail, deleting and
 * adding. Everything that reaches the server goes through the store, mocked here.
 */
const taskId = 't1' as TaskId

function makeTask(): Task {
  return {
    id: taskId,
    label: 'report-builder',
    title: 'Report builder',
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
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
    reviewNote: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    anchor: 'project:vega task:report-builder',
    updatedAt: '2026-10-01T10:00:00Z'
  }
}

function makeStep(id: string, position: number, state: TaskStepState, body: string | null = null): TaskStep {
  return {
    id: id as TaskStepId,
    taskId,
    title: `Step ${id}`,
    bodyMarkdown: body,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: state === 'CLAIMED' ? '2026-10-01T09:00:00Z' : null,
    doneAt: state === 'DONE' ? '2026-10-01T08:00:00Z' : null,
    position,
    flowX: null,
    flowY: null,
    maxVisits: null,
    passes: [],
    links: [],
    visits: [],
    createdAt: '2026-10-01T07:00:00Z',
    updatedAt: '2026-10-01T07:00:00Z'
  }
}

let pinia: Pinia

function seed(steps: TaskStep[], withTask = true) {
  const store = useConsoleStore()
  store.tasks = withTask ? [makeTask()] : []
  store.selectedTaskId = withTask ? taskId : null
  store.steps = steps
  store.isLoading = false
  store.acceptStep = vi.fn().mockResolvedValue(undefined)
  store.toggleStep = vi.fn().mockResolvedValue(undefined)
  store.sendBackStep = vi.fn().mockResolvedValue(undefined)
  store.reopenStep = vi.fn().mockResolvedValue(undefined)
  store.promoteStep = vi.fn().mockResolvedValue(undefined)
  store.removeStep = vi.fn().mockResolvedValue(undefined)
  store.moveStep = vi.fn().mockResolvedValue(undefined)
  store.saveStep = vi.fn().mockResolvedValue(undefined)
  store.addStep = vi.fn().mockResolvedValue(null)
  return store
}

function render() {
  return mount(StepsPane, {
    attachTo: document.body,
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        NotesButton: true,
        PlanHereButton: true,
        OpenTerminalButton: true,
        LaunchClaudeCodeButton: true,
        LogCommitButton: true,
        CommitReferenceList: true,
        AppConfirm: {
          template: '<div data-testid="confirm"><button data-testid="confirm-yes" @click="$emit(\'confirm\')" /></div>'
        }
      }
    }
  })
}

describe('StepsPane', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  afterEach(() => {
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  it('asks for a task when none is selected', () => {
    seed([], false)
    expect(render().text()).toContain('Pick a task to see what is left on it.')
  })

  it('offers the first step and the plan command on a task with no steps', () => {
    seed([])
    const pane = render()
    expect(pane.find('[data-testid="write-first-step"]').exists()).toBe(true)
    expect(pane.get('[data-testid="copy-plan-command"]').text()).toContain('project:vega task:report-builder plan')
    expect(pane.find('[data-testid="steps-ledger"]').exists()).toBe(false)
  })

  it('counts done, claimed and draft steps on the ledger', () => {
    seed([makeStep('a', 0, 'DONE'), makeStep('b', 1, 'CLAIMED'), makeStep('c', 2, 'OPEN'), makeStep('d', 3, 'DRAFT')])
    const pane = render()
    expect(pane.get('[data-testid="steps-count"]').text()).toBe('1/3')
    expect(pane.get('[data-testid="steps-claimed-count"]').text()).toBe('1 awaiting review')
    expect(pane.get('[data-testid="steps-draft-count"]').text()).toContain('1 in draft')
  })

  it('opens the first unfinished step on its own, and a title click switches to another', async () => {
    seed([makeStep('a', 0, 'DONE'), makeStep('b', 1, 'OPEN'), makeStep('c', 2, 'OPEN')])
    const pane = render()
    const rows = () => pane.findAll('[data-testid="step-row"]')
    expect(rows()[1]!.find('[data-testid="step-detail"]').exists()).toBe(true)

    await rows()[2]!.get('[data-testid="step-title"]').trigger('click')

    expect(rows()[1]!.find('[data-testid="step-detail"]').exists()).toBe(false)
    expect(rows()[2]!.find('[data-testid="step-detail"]').exists()).toBe(true)
  })

  it('ticks an open step and accepts a claimed one from the checkbox', async () => {
    const store = seed([makeStep('a', 0, 'OPEN'), makeStep('b', 1, 'CLAIMED')])
    const pane = render()
    const boxes = pane.findAll('[data-testid="step-checkbox"]')

    await boxes[0]!.trigger('click')
    await boxes[1]!.trigger('click')
    await flushPromises()

    expect(store.toggleStep).toHaveBeenCalledWith('a')
    expect(store.acceptStep).toHaveBeenCalledWith('b')
  })

  it('sends a claimed step back from the review bar its detail opens on', async () => {
    const store = seed([makeStep('b', 0, 'CLAIMED')])
    const pane = render()
    expect(pane.find('[data-testid="step-review-bar"]').exists()).toBe(false)
    await pane.get('[data-testid="step-title"]').trigger('click')

    await pane.get('[data-testid="step-send-back"]').trigger('click')
    await flushPromises()

    expect(store.sendBackStep).toHaveBeenCalledWith('b')
  })

  it('reopens a done step only on the second click', async () => {
    const store = seed([makeStep('a', 0, 'DONE')])
    const pane = render()
    await pane.get('[data-testid="step-title"]').trigger('click')

    await pane.get('[data-testid="step-reopen"]').trigger('click')
    expect(store.reopenStep).not.toHaveBeenCalled()
    expect(pane.get('[data-testid="step-reopen"]').text()).toBe('Confirm reopen')

    await pane.get('[data-testid="step-reopen"]').trigger('click')
    await flushPromises()
    expect(store.reopenStep).toHaveBeenCalledWith('a')
  })

  it('hides and shows the done steps', async () => {
    seed([makeStep('a', 0, 'DONE'), makeStep('b', 1, 'OPEN')])
    const pane = render()
    expect(pane.findAll('[data-testid="step-row"]')).toHaveLength(2)

    await pane.get('[data-testid="toggle-hide-done"]').trigger('click')
    expect(pane.findAll('[data-testid="step-row"]')).toHaveLength(1)

    await pane.get('[data-testid="toggle-hide-done"]').trigger('click')
    expect(pane.findAll('[data-testid="step-row"]')).toHaveLength(2)
  })

  it('promotes a draft and moves a checklist step down', async () => {
    const store = seed([makeStep('a', 0, 'OPEN'), makeStep('b', 1, 'OPEN'), makeStep('d', 2, 'DRAFT')])
    const pane = render()

    await pane.get('[data-testid="draft-promote"]').trigger('click')
    await pane.findAll('[data-testid="step-down"]')[0]!.trigger('click')
    await flushPromises()

    expect(store.promoteStep).toHaveBeenCalledWith('d')
    expect(store.moveStep).toHaveBeenCalledWith('a', 1)
  })

  it('deletes a step only once the confirmation is given', async () => {
    const store = seed([makeStep('a', 0, 'OPEN')])
    const pane = render()

    await pane.get('[data-testid="step-delete"]').trigger('click')
    expect(store.removeStep).not.toHaveBeenCalled()
    await pane.get('[data-testid="confirm-yes"]').trigger('click')
    await flushPromises()

    expect(store.removeStep).toHaveBeenCalledWith('a')
  })

  it('adds a draft with the detail template', async () => {
    const store = seed([makeStep('a', 0, 'OPEN')])
    const pane = render()

    await pane.get('[data-testid="new-step"]').setValue('  Export the report  ')
    await pane.get('form').trigger('submit')
    await flushPromises()

    expect(store.addStep).toHaveBeenCalledWith(taskId, 'Export the report', STEP_DETAIL_TEMPLATE)
  })

  it('saves an edited title a moment after the typing stops', async () => {
    vi.useFakeTimers()
    const store = seed([makeStep('a', 0, 'OPEN')])
    const pane = render()
    await pane.get('[data-testid="step-detail-write"]').trigger('click')

    await pane.get('[data-testid="step-title-field"]').setValue('Renamed')
    expect(store.saveStep).not.toHaveBeenCalled()
    vi.advanceTimersByTime(700)
    await flushPromises()

    expect(store.saveStep).toHaveBeenCalledWith('a', { title: 'Renamed', bodyMarkdown: '' })
  })
})
