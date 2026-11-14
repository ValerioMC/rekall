import { defineStore } from 'pinia'
import { ref } from 'vue'
import { loadPersistedSet, savePersistedSet } from '@/common/config/persisted-set'
import type { ProjectId } from '@/model/branded'

const KEY = 'rekall.recent.projects'
const KEPT = 5

/** The projects opened last, newest first, remembered per browser for the project switcher. */
export const useRecentProjectsStore = defineStore('recentProjects', () => {
  const ids = ref<readonly ProjectId[]>([...loadPersistedSet<ProjectId>(KEY)].reverse())

  function remember(projectId: ProjectId): void {
    ids.value = [projectId, ...ids.value.filter((id) => id !== projectId)].slice(0, KEPT)
    savePersistedSet(KEY, new Set([...ids.value].reverse()))
  }

  return { ids, remember }
})
