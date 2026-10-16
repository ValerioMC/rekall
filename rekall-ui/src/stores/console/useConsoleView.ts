import { computed } from 'vue'
import { compareText } from '@/features/catalog/lib/compare-text'
import type { Project, RekallDocument, Task } from '@/features/catalog/model/catalog'
import type { ConsoleState } from './console-state'

function matchesTask(task: Task, needle: string): boolean {
  if (!needle.trim()) return true
  const hay = `${task.projectLabel} ${task.projectTitle} ${task.label} ${task.title}`.toLowerCase()
  return needle
    .toLowerCase()
    .replace(/(project:|task:|company:)/g, ' ')
    .replace(/\//g, ' ')
    .trim()
    .split(/\s+/)
    .every((part) => hay.includes(part))
}

function matchesDocument(document: RekallDocument, needle: string): boolean {
  if (!needle.trim()) return true
  const hay = `${document.title} ${document.kind} ${document.bodyMarkdown}`.toLowerCase()
  return needle
    .toLowerCase()
    .trim()
    .split(/\s+/)
    .every((part) => hay.includes(part))
}

const byTitle = (a: { readonly title: string }, b: { readonly title: string }): number =>
  compareText(a.title, b.title)

/** What the navigator lists: the rows inside the company or project in scope that match the filter, by title. */
export function useConsoleView(state: ConsoleState) {
  const { companies, projects, tasks, documents, scopeCompany, scopeProject, navMode, filter } = state

  const projectInScope = (project: Project): boolean =>
    (scopeCompany.value === null || project.companyId === scopeCompany.value) &&
    (scopeProject.value === null || project.id === scopeProject.value)

  const inScope = (task: Task): boolean => {
    if (scopeProject.value !== null) return task.projectId === scopeProject.value
    if (scopeCompany.value === null) return true
    return projects.value.find((p) => p.id === task.projectId)?.companyId === scopeCompany.value
  }

  /** A note shows where its owner is in view: a global one everywhere, a company's under it and its projects. */
  const documentInScope = (document: RekallDocument): boolean => {
    const owner = document.scope
    if (owner.kind === 'GLOBAL' || (scopeCompany.value === null && scopeProject.value === null)) return true
    const companyInView =
      scopeProject.value !== null
        ? projects.value.find((project) => project.id === scopeProject.value)?.companyId
        : scopeCompany.value
    if (owner.kind === 'COMPANY') return owner.id === companyInView
    if (scopeProject.value !== null) return owner.id === scopeProject.value
    return projects.value.find((project) => project.id === owner.id)?.companyId === companyInView
  }

  const archivedProjectIds = computed(
    () => new Set(projects.value.filter((project) => project.status === 'ARCHIVED').map((project) => project.id))
  )

  /** An archived project's work stays out of the navigator unless the scope names that project. */
  const isShelved = (task: Task): boolean =>
    archivedProjectIds.value.has(task.projectId) && scopeProject.value !== task.projectId

  const visibleTasks = computed(() =>
    tasks.value
      .filter((task) => inScope(task) && !isShelved(task) && matchesTask(task, filter.value))
      .sort(byTitle)
  )

  const visibleDocuments = computed(() =>
    documents.value
      .filter((document) => documentInScope(document) && matchesDocument(document, filter.value))
      .sort(byTitle)
  )

  const scopedCompany = computed(
    () => companies.value.find((company) => company.id === scopeCompany.value) ?? null
  )

  const scopedProject = computed(
    () => projects.value.find((project) => project.id === scopeProject.value) ?? null
  )

  const recentDocuments = computed(() =>
    [...documents.value].sort((a, b) => b.updatedAt.localeCompare(a.updatedAt)).slice(0, 4)
  )

  const scopedProjects = computed(() => projects.value.filter(projectInScope))

  const scopePath = computed<string[]>(() => {
    if (!scopedCompany.value) return []
    return scopedProject.value
      ? [scopedCompany.value.name, scopedProject.value.title]
      : [scopedCompany.value.name]
  })

  const scopeName = computed(() => (scopePath.value.length === 0 ? 'All work' : scopePath.value.join(' / ')))

  const scopeAnchor = computed(() => {
    if (scopedProject.value) return scopedProject.value.anchor
    return scopedCompany.value ? `company:${scopedCompany.value.name}` : ''
  })

  /** How many filter matches the scope hides, and where they are, so the navigator can offer to widen it. */
  const elsewhere = computed(() => {
    if ((scopeCompany.value === null && scopeProject.value === null) || !filter.value.trim()) return null
    const outTasks = tasks.value.filter((t) => !inScope(t) && matchesTask(t, filter.value))
    const outDocs = documents.value.filter((d) => !documentInScope(d) && matchesDocument(d, filter.value))
    const count = navMode.value === 'tasks' ? outTasks.length : outDocs.length
    if (count === 0) return null
    const names = [
      ...new Set(
        navMode.value === 'tasks'
          ? outTasks.map((t) => (scopeProject.value ? t.projectTitle : t.companyName))
          : outDocs.flatMap((d) =>
              d.tasks.map((ref) => (scopeProject.value ? ref.projectTitle : ref.companyName))
            )
      )
    ]
    return { count, names }
  })

  return {
    inScope,
    visibleTasks,
    visibleDocuments,
    scopedCompany,
    scopedProject,
    scopedProjects,
    recentDocuments,
    scopePath,
    scopeName,
    scopeAnchor,
    elsewhere
  }
}
