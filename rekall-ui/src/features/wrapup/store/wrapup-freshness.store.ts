import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchWrapupFreshness } from '@/features/wrapup/api/wrapup-versions.api'
import type { WrapupFreshness } from '@/features/wrapup/model/wrapup-version'
import type { TaskId } from '@/model/branded'

/** What git said about the wrapups the console has shown, read again whenever a wrapup is written. */
export const useWrapupFreshnessStore = defineStore('wrapupFreshness', () => {
  const freshnessByTask = ref<Readonly<Record<string, WrapupFreshness>>>({})

  function freshnessOf(taskId: TaskId): WrapupFreshness {
    return freshnessByTask.value[taskId] ?? { state: 'unknown' }
  }

  async function load(taskId: TaskId): Promise<void> {
    freshnessByTask.value = { ...freshnessByTask.value, [taskId]: await fetchWrapupFreshness(taskId) }
  }

  return { freshnessOf, load }
})
