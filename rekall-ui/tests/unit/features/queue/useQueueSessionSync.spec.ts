import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { useQueueSessionSync } from '@/features/queue/composables/useQueueSessionSync'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { EMPTY_RUN_QUEUE, asRunQueueItemId, type RunQueueItem } from '@/features/queue/model/runQueue'
import { fetchTerminals } from '@/features/claude/api/terminal.api'
import { asTaskId } from '@/model/branded'

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

function item(state: RunQueueItem['state']): RunQueueItem {
  return {
    id: asRunQueueItemId('item-1'),
    taskId: asTaskId('task-1'),
    taskTitle: 'Report builder',
    taskLabel: 'report-builder',
    projectLabel: 'vega',
    anchor: 'project:vega task:report-builder',
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null,
    steps: [],
    waitingFor: []
  }
}

function mountSync(): void {
  mount(defineComponent({ setup: () => useQueueSessionSync(), template: '<div />' }))
}

describe('useQueueSessionSync', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(fetchTerminals).mockReset().mockResolvedValue([])
  })

  it('reads the sessions again when the queue starts running a task', async () => {
    mountSync()
    vi.mocked(fetchTerminals).mockClear()

    useRunQueueStore().apply({ ...EMPTY_RUN_QUEUE, items: [item('RUNNING')], updatedAt: '2026-09-23T10:00:05Z' })
    await nextTick()

    expect(fetchTerminals).toHaveBeenCalledTimes(1)
  })

  it('lists the session the queue opened, so the task row can show it', async () => {
    mountSync()
    const session = { id: 'term-9', taskId: 'task-1', live: true }
    vi.mocked(fetchTerminals).mockResolvedValue([session] as never)

    useRunQueueStore().apply({ ...EMPTY_RUN_QUEUE, items: [item('RUNNING')], updatedAt: '2026-09-23T10:00:05Z' })
    await nextTick()
    await Promise.resolve()

    expect(useTerminalStore().terminals).toEqual([session])
  })

  it('leaves the sessions alone when the queue changes without moving the item in hand', async () => {
    mountSync()
    useRunQueueStore().apply({ ...EMPTY_RUN_QUEUE, items: [item('RUNNING')], updatedAt: '2026-09-23T10:00:05Z' })
    await nextTick()
    vi.mocked(fetchTerminals).mockClear()

    useRunQueueStore().apply({ ...EMPTY_RUN_QUEUE, ceilingPercent: 80, items: [item('RUNNING')], updatedAt: '2026-09-23T10:00:09Z' })
    await nextTick()

    expect(fetchTerminals).not.toHaveBeenCalled()
  })
})
