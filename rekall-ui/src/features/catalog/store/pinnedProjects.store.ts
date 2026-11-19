import { defineStore } from 'pinia'
import { ref } from 'vue'
import { loadPersistedSet, savePersistedSet } from '@/common/config/persisted-set'
import type { ProjectId } from '@/model/branded'

const KEY = 'rekall.pinned.projects'
export const PINNED_LIMIT = 3

export type PinOutcome = 'pinned' | 'unpinned' | 'full'

/** The projects kept at the top of the project switcher, in the order they were pinned, remembered per browser. */
export const usePinnedProjectsStore = defineStore('pinnedProjects', () => {
  const ids = ref<readonly ProjectId[]>([...loadPersistedSet<ProjectId>(KEY)])

  function isPinned(projectId: ProjectId): boolean {
    return ids.value.includes(projectId)
  }

  function toggle(projectId: ProjectId): PinOutcome {
    if (isPinned(projectId)) {
      ids.value = ids.value.filter((id) => id !== projectId)
      savePersistedSet(KEY, new Set(ids.value))
      return 'unpinned'
    }
    if (ids.value.length >= PINNED_LIMIT) return 'full'
    ids.value = [...ids.value, projectId]
    savePersistedSet(KEY, new Set(ids.value))
    return 'pinned'
  }

  return { ids, isPinned, toggle }
})
