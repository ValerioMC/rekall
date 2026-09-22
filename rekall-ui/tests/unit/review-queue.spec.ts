import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import { reviewQueue } from '@/model/review'
import type { ReviewItem } from '@/model/review'
import { noticeFor, useClaimNotifications } from '@/composables/useClaimNotifications'
import type { Task, TaskStep, TaskStepState } from '@/model/catalog'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

const postNotice = vi.fn(async () => true)
vi.mock('@/common/native/notify', () => ({ postNotice: (...args: unknown[]) => postNotice(...(args as [])) }))

function task(id: string, title: string, review: Partial<Pick<Task, 'reviewState' | 'reviewActive' | 'claimedAt'>> = {}): Task {
  return {
    id: id as TaskId,
    label: id,
    title,
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
    projectRepoFolder: null,
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: review.reviewState ?? 'OPEN',
    reviewActive: review.reviewActive ?? false,
    claimedAt: review.claimedAt ?? null,
    acceptedAt: null,
    reviewNote: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    anchor: `project:vega task:${id}`,
    updatedAt: '2026-09-20T10:00:00Z'
  }
}

function step(id: string, taskId: string, state: TaskStepState, claimedAt: string | null = null): TaskStep {
  return {
    id: id as TaskStepId,
    taskId: taskId as TaskId,
    title: `Step ${id}`,
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt,
    doneAt: null,
    position: 0,
    createdAt: '2026-09-20T09:00:00Z',
    updatedAt: '2026-09-20T09:00:00Z'
  }
}

describe('the review queue', () => {
  it('holds claimed steps and claimed stepless tasks, the longest-waiting first', () => {
    const tasks = [
      task('checklist', 'With a checklist'),
      task('stepless', 'No checklist', { reviewState: 'CLAIMED', reviewActive: true, claimedAt: '2026-09-20T08:00:00Z' })
    ]
    const steps = [
      step('s1', 'checklist', 'CLAIMED', '2026-09-20T09:30:00Z'),
      step('s2', 'checklist', 'RUNNING'),
      step('s3', 'checklist', 'DONE')
    ]

    const queue = reviewQueue(tasks, steps)

    expect(queue.map((item) => item.key)).toEqual(['stepless', 's1'])
    expect(queue[1]).toMatchObject({ stepTitle: 'Step s1', taskTitle: 'With a checklist', stepId: 's1' })
    expect(queue[0]).toMatchObject({ stepId: null, taskTitle: 'No checklist' })
  })

  it('leaves out a task whose review line is not active, and a step whose task is gone', () => {
    const tasks = [task('t', 'Has steps', { reviewState: 'CLAIMED', reviewActive: false })]

    expect(reviewQueue(tasks, [step('orphan', 'missing', 'CLAIMED')])).toEqual([])
  })
})

describe('the claim notification', () => {
  const item = (key: string, stepTitle: string | null): ReviewItem => ({
    key,
    taskId: 't1' as TaskId,
    stepId: stepTitle ? (key as TaskStepId) : null,
    taskTitle: 'Report builder',
    stepTitle,
    anchor: 'project:vega task:t1',
    claimedAt: null
  })

  it('names one step with its task, one task alone, and counts several', () => {
    expect(noticeFor([item('s1', 'Wire the export')])).toEqual({
      title: 'A step is waiting for review',
      body: 'Wire the export — Report builder'
    })
    expect(noticeFor([item('t1', null)])).toEqual({ title: 'A task is waiting for review', body: 'Report builder' })
    expect(noticeFor([item('s1', 'A'), item('s2', 'B')]).title).toBe('2 pieces of work are waiting for review')
  })

  describe('when to post', () => {
    let hasFocus: ReturnType<typeof vi.spyOn>

    beforeEach(() => {
      postNotice.mockClear()
      hasFocus = vi.spyOn(document, 'hasFocus').mockReturnValue(false)
    })

    afterEach(() => hasFocus.mockRestore())

    it('never for what was already waiting when the console loaded, only for what arrives after', async () => {
      const queue = ref<ReviewItem[]>([item('old', 'Old')])
      const loading = ref(true)
      useClaimNotifications(queue, loading)

      loading.value = false
      await nextTick()
      expect(postNotice).not.toHaveBeenCalled()

      queue.value = [item('old', 'Old'), item('new', 'New')]
      await nextTick()
      expect(postNotice).toHaveBeenCalledWith({ title: 'A step is waiting for review', body: 'New — Report builder' })
    })

    it('not while the console is the window in front', async () => {
      hasFocus.mockReturnValue(true)
      const queue = ref<ReviewItem[]>([])
      const loading = ref(false)
      useClaimNotifications(queue, loading)

      queue.value = [item('new', 'New')]
      await nextTick()

      expect(postNotice).not.toHaveBeenCalled()
    })
  })
})
