import { computed } from 'vue'
import { compareText } from '@/features/catalog/lib/compare-text'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { ConsoleState } from './console-state'

const byTitle = (a: { readonly title: string }, b: { readonly title: string }): number =>
  compareText(a.title, b.title)

/** The rows inside the company or project in scope; `visibleTasks` lists them by title. */
export function useConsoleView(state: ConsoleState) {
  const { companies, projects, tasks, scopeCompany, scopeProject, hideQuickTasks } = state

  const projectInScope = (project: Project): boolean =>
    (scopeCompany.value === null || project.companyId === scopeCompany.value) &&
    (scopeProject.value === null || project.id === scopeProject.value)

  const inScope = (task: Task): boolean => {
    if (scopeProject.value !== null) return task.projectId === scopeProject.value
    if (scopeCompany.value === null) return true
    return projects.value.find((p) => p.id === task.projectId)?.companyId === scopeCompany.value
  }

  const archivedProjectIds = computed(
    () => new Set(projects.value.filter((project) => project.status === 'ARCHIVED').map((project) => project.id))
  )

  /** An archived project's work stays out of the navigator unless the scope names that project. */
  const isShelved = (task: Task): boolean =>
    archivedProjectIds.value.has(task.projectId) && scopeProject.value !== task.projectId

  const scopedTasks = computed(() =>
    tasks.value.filter((task) => inScope(task) && !isShelved(task) && !(hideQuickTasks.value && task.quick))
  )

  const visibleTasks = computed(() =>
    [...scopedTasks.value].sort(byTitle)
  )

  const scopedCompany = computed(
    () => companies.value.find((company) => company.id === scopeCompany.value) ?? null
  )

  const scopedProject = computed(
    () => projects.value.find((project) => project.id === scopeProject.value) ?? null
  )

  const scopedProjects = computed(() => projects.value.filter(projectInScope))

  const scopePath = computed<string[]>(() => {
    if (!scopedCompany.value) return []
    return scopedProject.value
      ? [scopedCompany.value.name, scopedProject.value.title]
      : [scopedCompany.value.name]
  })

  const scopeAnchor = computed(() => {
    if (scopedProject.value) return scopedProject.value.anchor
    return scopedCompany.value ? `company:${scopedCompany.value.name}` : ''
  })

  return {
    inScope,
    scopedTasks,
    visibleTasks,
    scopedCompany,
    scopedProject,
    scopedProjects,
    scopePath,
    scopeAnchor
  }
}
