import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import ReviewQueueButton from '@/components/console/ReviewQueueButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task, TaskStep } from '@/model/catalog'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

const report: Task = {
  id: 't1' as TaskId,
  label: 'report-builder',
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
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
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
  createdAt: '2026-09-22T08:00:00Z',
  updatedAt: '2026-09-22T09:00:00Z'
}

describe('the Review button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('is not there while nothing waits for review', () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [{ ...claimedStep, state: 'RUNNING' }]

    const wrapper = mount(ReviewQueueButton)

    expect(wrapper.find('[data-testid="review-queue-open"]').exists()).toBe(false)
  })

  it('counts what waits, lists it, and opens the one picked', async () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [claimedStep]
    store.openReviewItem = vi.fn()

    const wrapper = mount(ReviewQueueButton, { attachTo: document.body })
    expect(wrapper.find('[data-testid="review-queue-count"]').text()).toBe('1')

    await wrapper.find('[data-testid="review-queue-open"]').trigger('click')
    const row = wrapper.find('[data-testid="review-queue-item"]')
    expect(row.text()).toContain('Wire the export')
    expect(row.text()).toContain('Report builder')

    await row.trigger('click')
    expect(store.openReviewItem).toHaveBeenCalledWith(expect.objectContaining({ stepId: 's1', taskId: 't1' }))
    expect(wrapper.find('[data-testid="review-queue"]').exists()).toBe(false)
    wrapper.unmount()
  })
})
