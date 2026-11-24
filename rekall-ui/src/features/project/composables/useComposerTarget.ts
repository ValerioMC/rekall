import { computed, ref, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

export interface ComposerTarget {
  readonly target: ComputedRef<Project | null>
  readonly choices: ComputedRef<readonly Project[]>
  choose: (projectId: ProjectId) => void
}

/** The project a new task goes to: the one in view, or, across projects, the one picked, else the one worked on most recently. */
export function useComposerTarget(): ComposerTarget {
  const store = useConsoleStore()
  const { projectInView } = storeToRefs(useProjectTasksStore())
  const chosenId = ref<ProjectId | null>(null)

  const choices = computed(() => (projectInView.value === null ? store.scopedProjects : []))
  const recentProjectId = computed(
    () => [...store.scopedTasks].sort((first, second) => second.updatedAt.localeCompare(first.updatedAt))[0]?.projectId ?? null
  )
  const target = computed(() => {
    if (projectInView.value !== null) return store.projects.find((project) => project.id === projectInView.value) ?? null
    const preferred = chosenId.value ?? recentProjectId.value
    return store.scopedProjects.find((project) => project.id === preferred) ?? store.scopedProjects[0] ?? null
  })

  function choose(projectId: ProjectId): void {
    chosenId.value = projectId
  }

  return { target, choices, choose }
}
