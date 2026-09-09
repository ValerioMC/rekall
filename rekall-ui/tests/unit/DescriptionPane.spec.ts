import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import DescriptionPane from '@/components/console/DescriptionPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task, TaskStepState } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * The description's own review bar, shown only while the task has no checklist. It has to give
 * the same OPEN -> RUNNING -> CLAIMED -> DONE affordances a step does: while a session is on a
 * stepless task the bar still offers Accept, because otherwise the running state has no console
 * exit and the wrapup path is the only way forward.
 */
const rekall = 'p1' as ProjectId
const taskId = 't1' as TaskId

function makeTask(reviewState: TaskStepState): Task {
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
    stepsDone: 0,
    hasWrapup: false,
    reviewState,
    reviewActive: true,
    claimedAt: null,
    acceptedAt: null,
    reviewNote: null,
    anchor: 'project:rekall task:application-improvements',
    updatedAt: '2026-09-09T10:00:00Z'
  }
}

function seed(reviewState: TaskStepState) {
  const store = useConsoleStore()
  store.tasks = [makeTask(reviewState)]
  store.selectedTaskId = taskId
  store.isLoading = false
  store.acceptTask = vi.fn().mockResolvedValue(undefined)
  store.sendBackTask = vi.fn().mockResolvedValue(undefined)
  store.saveTaskDescription = vi.fn().mockResolvedValue(undefined)
  return store
}

let pinia: Pinia

function render() {
  return mount(DescriptionPane, {
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        ClaudeSessionLauncher: true,
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

  it('offers Accept while a session has the stepless task running, without a Send back', async () => {
    const store = seed('RUNNING')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="description-review-bar"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="description-send-back"]').exists()).toBe(false)

    await wrapper.get('[data-testid="description-accept"]').trigger('click')
    expect(store.acceptTask).toHaveBeenCalledWith(taskId)
  })

  it('keeps Accept and Send back once a wrapup has claimed the task', async () => {
    seed('CLAIMED')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="description-accept"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="description-send-back"]').exists()).toBe(true)
  })

  it('shows no review bar while the stepless task is still open', async () => {
    seed('OPEN')
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="description-review-bar"]').exists()).toBe(false)
  })
})
