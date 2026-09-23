import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { EMPTY_RUN_QUEUE } from '@/model/runQueue'

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
})
