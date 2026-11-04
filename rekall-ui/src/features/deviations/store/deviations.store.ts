import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchDeviations } from '@/features/deviations/api/deviations.api'
import type { TaskDeviation } from '@/features/deviations/model/deviation'
import type { TaskId } from '@/model/branded'

/** The deviations of each task the console has shown, read when it is first shown and when a session writes one. */
export const useDeviationsStore = defineStore('deviations', () => {
  const byTask = ref<Readonly<Record<string, readonly TaskDeviation[]>>>({})

  function of(taskId: TaskId): readonly TaskDeviation[] {
    return byTask.value[taskId] ?? []
  }

  async function load(taskId: TaskId): Promise<void> {
    byTask.value = { ...byTask.value, [taskId]: await fetchDeviations(taskId) }
  }

  function isKnown(taskId: TaskId): boolean {
    return taskId in byTask.value
  }

  return { of, load, isKnown }
})
