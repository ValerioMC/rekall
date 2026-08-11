import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchEnvironments, fetchProjects } from '@/api/catalog.api'
import type { Environment, Project } from '@/model/catalog'

/**
 * Projects and environments, kept in one place because both are needed by the sidebar and by
 * every task form. Tasks are not cached: they are always scoped to a project and a stale list
 * there is more confusing than a second request.
 */
export const useCatalogStore = defineStore('catalog', () => {
  const projects = ref<Project[]>([])
  const environments = ref<Environment[]>([])
  const isLoading = ref(false)

  const activeProjects = computed(() => projects.value.filter((p) => p.status === 'ACTIVE'))

  async function load(): Promise<void> {
    isLoading.value = true
    try {
      const [loadedProjects, loadedEnvironments] = await Promise.all([
        fetchProjects(),
        fetchEnvironments()
      ])
      projects.value = loadedProjects
      environments.value = loadedEnvironments
    } finally {
      isLoading.value = false
    }
  }

  return { projects, environments, activeProjects, isLoading, load }
})
