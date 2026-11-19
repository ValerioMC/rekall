import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ReviewQueueButton from '@/features/catalog/components/ReviewQueueButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

const report: Task = {
  id: 't1' as TaskId,
  code: 'report-builder', label: 'report-builder',
  title: 'Report builder',
  status: 'IN_PROGRESS',
  description: null,
  projectId: 'p1' as ProjectId,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 1,
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
  epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
  anchor: 'project:vega task:report-builder',
  updatedAt: '2026-09-22T09:00:00Z'
}

const claimedStep: TaskStep = {
  id: 's1' as TaskStepId,
  taskId: report.id,
  title: 'Wire the export',
  bodyMarkdown: null,
  state: 'CLAIMED',
  done: false,
  runningAt: null,
  claimedAt: '2026-09-22T09:00:00Z',
  doneAt: null,
  position: 0,
  flowX: null,
  flowY: null,
  maxVisits: null,
  startsSession: false,
  connectionId: null,
  passes: [],
    links: [],
    visits: [],
  createdAt: '2026-09-22T08:00:00Z',
  updatedAt: '2026-09-22T09:00:00Z'
}

const stub = { template: '<div />' }

function mountButton() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/review/:key?', name: 'review', component: stub }]
  })
  return mount(ReviewQueueButton, { global: { plugins: [router] } })
}

describe('the Review button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('is not there while nothing waits for review', () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [{ ...claimedStep, state: 'RUNNING' }]

    expect(mountButton().find('[data-testid="review-queue-open"]').exists()).toBe(false)
  })

  it('counts what waits and links to the Review screen', () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [claimedStep]

    const wrapper = mountButton()

    expect(wrapper.find('[data-testid="review-queue-count"]').text()).toBe('1')
    expect(wrapper.get('[data-testid="review-queue-open"]').attributes('href')).toBe('/review')
  })
})
