import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { EMPTY_RUN_QUEUE, asRunQueueItemId, type RunQueueItem } from '@/features/queue/model/runQueue'
import { runTaskInQueue } from '@/features/queue/api/runQueue.api'
import { asTaskId } from '@/model/branded'

vi.mock('@/features/queue/api/runQueue.api', () => ({
  runTaskInQueue: vi.fn()
}))

describe('the run queue store', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('takes a newer queue and drops an older one, so a slow reply cannot rewind an event', () => {
    const store = useRunQueueStore()
    store.apply({ ...EMPTY_RUN_QUEUE, state: 'RUNNING', updatedAt: '2026-09-23T10:00:05Z' })

    store.apply({ ...EMPTY_RUN_QUEUE, state: 'IDLE', updatedAt: '2026-09-23T10:00:01Z' })

    expect(store.queue.state).toBe('RUNNING')

    store.apply({ ...EMPTY_RUN_QUEUE, state: 'HOLDING', updatedAt: '2026-09-23T10:00:09Z' })
    expect(store.queue.state).toBe('HOLDING')
    expect(store.loaded).toBe(true)
  })

  it('applies the queue a run in queue answers with', async () => {
    const store = useRunQueueStore()
    vi.mocked(runTaskInQueue).mockResolvedValue({
      ...EMPTY_RUN_QUEUE,
      state: 'RUNNING',
      updatedAt: '2026-09-23T10:00:05Z'
    })

    await store.runInQueue(asTaskId('task-1'))

    expect(runTaskInQueue).toHaveBeenCalledWith('task-1')
    expect(store.queue.state).toBe('RUNNING')
  })

  it('drops an older queue answered to a run in queue', async () => {
    const store = useRunQueueStore()
    store.apply({ ...EMPTY_RUN_QUEUE, state: 'RUNNING', updatedAt: '2026-09-23T10:00:05Z' })
    vi.mocked(runTaskInQueue).mockResolvedValue({
      ...EMPTY_RUN_QUEUE,
      state: 'IDLE',
      updatedAt: '2026-09-23T10:00:01Z'
    })

    await store.runInQueue(asTaskId('task-1'))

    expect(store.queue.state).toBe('RUNNING')
  })

  it('exposes the running task step progress and moves it with each frame', () => {
    const store = useRunQueueStore()
    const running: RunQueueItem = {
      id: asRunQueueItemId('item-1'),
      taskId: asTaskId('task-1'),
      taskTitle: 'Task',
      taskCode: 'task',
      projectLabel: 'vega',
      anchor: 'project:vega task:task',
      position: 0,
      state: 'RUNNING',
      detail: null,
      startedAt: null,
      finishedAt: null,
      steps: [
        { title: 'One', state: 'RUNNING', current: true },
        { title: 'Two', state: 'OPEN', current: false }
      ],
      waitingFor: []
    }

    store.apply({ ...EMPTY_RUN_QUEUE, items: [running], updatedAt: '2026-09-23T10:00:01Z' })
    expect(store.progress.steps).toEqual({ total: 2, settled: 0, currentIndex: 1, currentTitle: 'One' })

    const claimed: RunQueueItem = {
      ...running,
      steps: [
        { title: 'One', state: 'CLAIMED', current: false },
        { title: 'Two', state: 'RUNNING', current: true }
      ]
    }
    store.apply({ ...EMPTY_RUN_QUEUE, items: [claimed], updatedAt: '2026-09-23T10:00:02Z' })

    expect(store.progress.steps).toEqual({ total: 2, settled: 1, currentIndex: 2, currentTitle: 'Two' })
  })
})
