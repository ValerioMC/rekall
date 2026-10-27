import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchEpicActivity } from '@/features/catalog/api/catalog.api'
import type { EpicActivity } from '@/features/catalog/model/epic'
import type { TaskId } from '@/model/branded'

/**
 * What moved lately on the tasks of each epic whose dashboard has been opened. A feed is fetched
 * when the dashboard opens and again whenever one of its tasks or their steps changes.
 */
export const useEpicActivityStore = defineStore('epicActivity', () => {
  const byEpic = ref<Readonly<Record<string, readonly EpicActivity[]>>>({})

  function activityOf(epicId: TaskId): readonly EpicActivity[] {
    return byEpic.value[epicId] ?? []
  }

  function isLoaded(epicId: TaskId): boolean {
    return epicId in byEpic.value
  }

  async function load(epicId: TaskId): Promise<void> {
    byEpic.value = { ...byEpic.value, [epicId]: await fetchEpicActivity(epicId) }
  }

  return { activityOf, isLoaded, load }
})
