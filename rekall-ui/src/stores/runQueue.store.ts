import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  clearSettledItems,
  dequeueItem,
  enqueueTask,
  fetchRunQueue,
  moveQueueItem,
  saveRunQueueSettings,
  startRunQueue,
  stopRunQueue
} from '@/api/runQueue.api'
import {
  EMPTY_RUN_QUEUE,
  runProgress,
  type RunQueue,
  type RunQueueItemId,
  type RunQueueSettings
} from '@/model/runQueue'
import type { TaskId } from '@/model/branded'

/**
 * The run queue as the console holds it. The server is the only place it runs, so every action
 * sends the change and takes back the whole queue; the `run-queue` SSE event keeps it current
 * between actions. A reply older than what is held is dropped, so a slow response cannot rewind
 * a newer event.
 */
export const useRunQueueStore = defineStore('runQueue', () => {
  const queue = ref<RunQueue>(EMPTY_RUN_QUEUE)
  const loaded = ref(false)
  const panelOpen = ref(false)

  const progress = computed(() => runProgress(queue.value))

  function apply(next: RunQueue): void {
    if (Date.parse(next.updatedAt) < Date.parse(queue.value.updatedAt)) return
    queue.value = next
    loaded.value = true
  }

  async function load(): Promise<void> {
    apply(await fetchRunQueue())
  }

  async function saveSettings(settings: RunQueueSettings): Promise<void> {
    apply(await saveRunQueueSettings(settings))
  }

  async function add(taskId: TaskId): Promise<void> {
    apply(await enqueueTask(taskId))
  }

  async function remove(itemId: RunQueueItemId): Promise<void> {
    apply(await dequeueItem(itemId))
  }

  async function move(itemId: RunQueueItemId, index: number): Promise<void> {
    apply(await moveQueueItem(itemId, index))
  }

  async function clearSettled(): Promise<void> {
    apply(await clearSettledItems())
  }

  async function start(startAt: string | null): Promise<void> {
    apply(await startRunQueue(startAt))
  }

  async function stop(): Promise<void> {
    apply(await stopRunQueue())
  }

  function openPanel(): void {
    panelOpen.value = true
  }

  function closePanel(): void {
    panelOpen.value = false
  }

  return {
    queue,
    loaded,
    panelOpen,
    progress,
    apply,
    load,
    saveSettings,
    add,
    remove,
    move,
    clearSettled,
    start,
    stop,
    openPanel,
    closePanel
  }
})
