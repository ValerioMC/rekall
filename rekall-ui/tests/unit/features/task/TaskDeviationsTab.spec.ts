import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskDeviationsTab from '@/features/task/components/TaskDeviationsTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const push = vi.fn()

vi.mock('vue-router', () => ({ useRouter: () => ({ push }), useRoute: () => ({ params: {} }) }))

const taskId = 't1' as TaskId

function makeTask(): Task {
  return {
    id: taskId, code: 'RK-1', label: 'one', title: 'Task one', status: 'IN_PROGRESS', description: null,
    projectId: 'p1' as ProjectId, projectLabel: 'vega', projectTitle: 'Vega', companyName: 'Acme', projectRepoFolder: '/work/vega',
    documentCount: 0, stepCount: 0, stepsDone: 0, draftStepCount: 0, hasWrapup: false, reviewState: 'OPEN', reviewActive: false,
    claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: null,
    releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null,
    trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null,
    pullRequestFailure: null, pullRequestMergedAt: null, anchor: 'task:RK-1', updatedAt: '2026-10-01T10:00:00Z'
  }
}

let pinia: Pinia

describe('TaskDeviationsTab', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    push.mockClear()
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('opens on the first deviation and says nothing about closing back to a pane', async () => {
    const store = useConsoleStore()
    store.tasks = [makeTask()]
    store.selectedTaskId = taskId
    store.isLoading = false
    const deviations = useDeviationsStore()
    deviations.load = vi.fn().mockResolvedValue(undefined)
    deviations.of = vi.fn().mockReturnValue([
      { id: 'd1', taskId, stepNumber: 4, stepTitle: 'Build the ledger', bodyMarkdown: 'planned', createdAt: '2026-10-01T09:00:00Z', updatedAt: '2026-10-01T09:00:00Z' },
      { id: 'd2', taskId, stepNumber: 6, stepTitle: 'Board', bodyMarkdown: 'planned', createdAt: '2026-10-01T09:00:00Z', updatedAt: '2026-10-01T09:00:00Z' }
    ])
    deviations.isKnown = vi.fn().mockReturnValue(true)

    const tab = mount(TaskDeviationsTab, {
      props: { task: makeTask() },
      attachTo: document.body,
      global: { plugins: [pinia], stubs: { AppMarkdownEditor: true } }
    })
    await flushPromises()

    expect(store.selectedDeviationStep).toBe(4)
    expect(tab.get('[data-testid="deviation-heading"]').text()).toContain('Step 4')
    expect(tab.find('[data-testid="deviation-close"]').exists()).toBe(false)
  })
})
